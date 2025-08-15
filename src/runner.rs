use crate::error::ImageProcessError;
use crate::options::{CliOptions, ResizeArgs, ResizeRule};
use caesium::parameters::CSParameters;
use caesium::SupportedFileTypes;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RunConfiguration {
    target_format: Option<SupportedFileTypes>,
    caesium_parameters: CSParameters,

    options: CliOptions,
}

impl From<CliOptions> for RunConfiguration {
    fn from(options: CliOptions) -> Self {
        Self {
            target_format: options.target_format.map_or(None, |v| Some(v.into())),
            caesium_parameters: options.clone().into(),
            options,
        }
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
        let options = &self.run_configuration.options;

        let input = &options.input;
        if !input.exists() {
            return Err(ImageProcessError::new(format!(
                "File or dir not exists: {}",
                input.to_string_lossy()
            )));
        }

        let output = options.output.clone();

        // Just run once for file input.
        if input.is_file() {
            let output_file = if output.is_dir() {
                Self::make_path(&input, &output, &options.prefix, &options.suffix)
            } else {
                output
            };

            return Self::run_process(&input, &output_file, run_configuration);
        }

        // Batch process

        // If input is dir, the output should be also a dir
        if !output.exists() {
            fs::create_dir_all(&output).map_err(|e| ImageProcessError::new(e.to_string()))?
        } else if output.is_file() {
            return Err(ImageProcessError::new(format!(
                "When input is a dir, output should also be a dir, but given a file: {}",
                output.to_str().unwrap_or("")
            )));
        }

        let input_files: Vec<PathBuf> = fs::read_dir(&input)?
            .filter_map(|e| e.ok())
            .map(|dir_entry| dir_entry.path())
            .filter_map(|path| if path.is_file() { Some(path) } else { None })
            .collect();

        let output_dir = output;
        for input_file in input_files {
            let output_file =
                Self::make_path(&input_file, &output_dir, &options.prefix, &options.suffix);

            let result = Self::run_process(&input_file, &output_file, run_configuration);
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

    fn make_path(
        input_file: &PathBuf,
        output_dir: &PathBuf,
        prefix: &Option<String>,
        suffix: &Option<String>,
    ) -> PathBuf {
        input_file
            .file_name()
            .map(|filename| filename.to_string_lossy().to_string())
            .map(|filename| match prefix {
                None => filename,
                Some(prefix) => format!("{}{}", prefix, filename),
            })
            .map(|filename| match suffix {
                None => filename,
                Some(suffix) => {
                    let parted_filename = Self::get_parted_filename(&filename);
                    match parted_filename.1 {
                        None => format!("{}{}", filename, suffix),
                        Some(ext) => format!("{}{}.{}", parted_filename.0, suffix, ext),
                    }
                }
            })
            .map(|filename| output_dir.join(filename))
            .unwrap()
    }

    fn get_parted_filename(filename: &String) -> (String, Option<String>) {
        if filename == "" {
            return ("".to_string(), None);
        }

        let path = Path::new(filename);
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap();
        let ext = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        (stem, ext)
    }

    fn run_process(
        input_file: &PathBuf,
        output_file: &PathBuf,
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
                // origin_data.clone()...?
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

        Ok(fs::write(output_file, &compressed).map_err(|e| ImageProcessError::from(e))?)
    }

    fn set_scaled_size(
        caesium_parameters: &mut CSParameters,
        resize_args: &ResizeArgs,
        data: &Vec<u8>,
    ) -> Result<(), ImageProcessError> {
        let image_size =
            imagesize::blob_size(data).map_err(|e| ImageProcessError::new(e.to_string()))?;

        let origin_width = image_size.width;
        let origin_height = image_size.height;

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
                    caesium_parameters.height = origin_height as u32;
                }
            }
            ResizeRule::Height => {
                caesium_parameters.height = resize_args.height as u32;
                if !resize_args.keep_aspect_ratio {
                    caesium_parameters.width = origin_width as u32;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
