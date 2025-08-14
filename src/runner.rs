use crate::error::{IOError, ImageProcessError};
use crate::options::{CliOptions, ResizeArgs, ResizeRule};
use caesium::parameters::CSParameters;
use caesium::SupportedFileTypes;
use image::ImageReader;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

pub struct RunConfiguration {
    input_files: Vec<PathBuf>,
    output_file_or_dir: PathBuf,

    target_format: Option<SupportedFileTypes>,
    caesium_parameters: CSParameters,

    options: CliOptions,
}

impl TryFrom<CliOptions> for RunConfiguration {
    type Error = IOError;

    fn try_from(options: CliOptions) -> Result<Self, Self::Error> {
        let mut run_configuration = Self {
            input_files: Vec::new(),
            output_file_or_dir: PathBuf::new(),

            target_format: None,
            caesium_parameters: options.clone().into(),

            options,
        };

        let opt = run_configuration.options.clone();

        let input = opt.input;
        if input.is_file() {
            run_configuration.input_files.push(input.clone());
        } else {
            for entry in fs::read_dir(&input)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    run_configuration.input_files.push(path);
                }
            }
        }

        let output = opt.output.map_or(input.clone(), |output| output);
        if run_configuration.input_files.len() > 1 {
            if output.exists() {
                if !output.is_dir() {
                    return Err(IOError::new(format!(
                        "When input is a dir, output should also be a dir, but given a file: {}",
                        output.to_str().unwrap_or("")
                    )));
                }
            } else {
               fs::create_dir_all(&run_configuration.output_file_or_dir)?
            }
        }

        run_configuration.output_file_or_dir = output;

        run_configuration.target_format = opt.target_format.map_or(None, |v| Some(v.into()));

        Ok(run_configuration)
    }
}

pub struct Runner {
    run_configuration: RunConfiguration,
}

impl From<RunConfiguration> for Runner {
    fn from(run_configuration: RunConfiguration) -> Self {
        Self { run_configuration }
    }
}

impl Runner {
    pub fn run(&self) -> Result<(), ImageProcessError> {
        let run_configuration = &self.run_configuration;

        let input_files = &run_configuration.input_files;
        for input_file in input_files {
            let result = Self::run_process(input_file, run_configuration);
            if let Err(err) = result {
                if !run_configuration.options.continue_on_error {
                    return Err(err);
                } else {
                    eprintln!("ImageProcessError of file '{:?}': {}", input_file, err);
                }
            }
        }

        Ok(())
    }

    fn run_process(
        input_file: &PathBuf,
        run_configuration: &RunConfiguration,
    ) -> Result<(), ImageProcessError> {
        let options = &run_configuration.options;

        let origin_data = fs::read(input_file).map_err(|e| ImageProcessError::from(e))?;

        let mut caesium_parameters = run_configuration.caesium_parameters.clone();

        let resize_args = &options.resize_args;
        if resize_args.rule != ResizeRule::NoResize {
            Self::set_scaled_size(&mut caesium_parameters, resize_args, &origin_data)?;
        }

        let compressed = match &run_configuration.target_format {
            None => caesium::compress_in_memory(origin_data, &caesium_parameters),
            Some(format) => {
                // file.clone()...?
                let convert_result = caesium::convert_in_memory(
                    origin_data.clone(),
                    &caesium_parameters,
                    format.clone(),
                );

                // code == 10407: output format same to the origin, just compress it
                if let Err(err) = &convert_result
                    && err.code == 10407
                {
                    caesium::compress_in_memory(origin_data, &caesium_parameters)
                } else {
                    convert_result
                }
            }
        };

        let compressed = compressed?;

        if options.delete_origin {
            fs::remove_file(&input_file).map_err(|e| ImageProcessError::from(e))?;
        }

        let output_file_or_dir = &run_configuration.output_file_or_dir;
        let output_file = if output_file_or_dir.is_dir() {
            input_file
                .file_name()
                .map(|filename| filename.to_string_lossy().to_string())
                .map(|filename| match &options.prefix {
                    Some(prefix) => format!("{}{}", prefix, filename),
                    None => filename.to_string(),
                })
                .map(|filename| output_file_or_dir.join(filename))
                .unwrap()
        } else {
            output_file_or_dir.clone()
        };

        Ok(fs::write(output_file, &compressed).map_err(|e| ImageProcessError::from(e))?)
    }

    fn set_scaled_size(
        caesium_parameters: &mut CSParameters,
        resize_args: &ResizeArgs,
        file: &Vec<u8>,
    ) -> Result<(), ImageProcessError> {
        let image_reader = ImageReader::new(Cursor::new(file));
        let image = image_reader.with_guessed_format()?.decode();
        let image = image.map_err(|e| ImageProcessError::new(e.to_string()))?;

        let origin_width = image.width();
        let origin_height = image.height();

        match resize_args.rule {
            ResizeRule::Size => {
                caesium_parameters.width = resize_args.width as u32;
                caesium_parameters.height = resize_args.height as u32;
            }
            ResizeRule::Scale => {
                if resize_args.ratio != 0.0 {
                    caesium_parameters.width = (origin_width as f32 * resize_args.ratio) as u32;
                    caesium_parameters.height = (origin_height as f32 * resize_args.ratio) as u32;
                } else {
                    caesium_parameters.width = (origin_width as f32 * resize_args.width) as u32;
                    caesium_parameters.height = (origin_height as f32 * resize_args.height) as u32;
                }
            }
            ResizeRule::ShortEdge | ResizeRule::LongEdge => {
                let set_width_to_edge_size = if resize_args.rule == ResizeRule::ShortEdge {
                    origin_width < origin_height
                } else {
                    origin_width > origin_height
                };

                if set_width_to_edge_size {
                    caesium_parameters.width = resize_args.edge_size;
                    if resize_args.keep_aspect_ratio {
                        let scale_ratio = resize_args.edge_size as f32 / origin_width as f32;
                        caesium_parameters.height = (origin_height as f32 * scale_ratio) as u32;
                    }
                } else {
                    caesium_parameters.height = resize_args.edge_size;
                    if resize_args.keep_aspect_ratio {
                        let scale_ratio = resize_args.edge_size as f32 / origin_height as f32;
                        caesium_parameters.width = (origin_width as f32 * scale_ratio) as u32;
                    }
                }
            }
            ResizeRule::Width => {
                caesium_parameters.width = resize_args.width as u32;
                if !resize_args.keep_aspect_ratio {
                    caesium_parameters.height = origin_height;
                }
            }
            ResizeRule::Height => {
                caesium_parameters.height = resize_args.height as u32;
                if !resize_args.keep_aspect_ratio {
                    caesium_parameters.width = origin_width;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
