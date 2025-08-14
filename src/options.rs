use crate::error::ValueParseError;
use caesium;
use caesium::parameters as cs_params;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

fn parse_kv(input: &str) -> HashMap<&str, &str> {
    input
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();
            if key.is_empty() {
                None
            } else {
                Some((key, value))
            }
        })
        .collect()
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum OutputFormatTypes {
    Jpeg,
    Png,
    Gif,
    WebP,
    Tiff,
}

impl str::FromStr for OutputFormatTypes {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(Self::Jpeg),
            "png" => Ok(Self::Png),
            "gif" => Ok(Self::Gif),
            "webp" => Ok(Self::WebP),
            "tiff" => Ok(Self::Tiff),
            _ => Err(ValueParseError::new(format!("Invalid value '{}'", s))),
        }
    }
}

impl From<OutputFormatTypes> for caesium::SupportedFileTypes {
    fn from(v: OutputFormatTypes) -> Self {
        match v {
            OutputFormatTypes::Jpeg => caesium::SupportedFileTypes::Jpeg,
            OutputFormatTypes::Png => caesium::SupportedFileTypes::Png,
            OutputFormatTypes::Gif => caesium::SupportedFileTypes::Gif,
            OutputFormatTypes::WebP => caesium::SupportedFileTypes::WebP,
            OutputFormatTypes::Tiff => caesium::SupportedFileTypes::Tiff,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ResizeRule {
    NoResize,
    Size,
    Scale,
    ShortEdge,
    LongEdge,
    Width,
    Height,
}

impl str::FromStr for ResizeRule {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "no_resize" => Ok(Self::NoResize),
            "size" => Ok(ResizeRule::Size),
            "scale" => Ok(ResizeRule::Scale),
            "short_edge" => Ok(ResizeRule::ShortEdge),
            "long_edge" => Ok(ResizeRule::LongEdge),
            "width" => Ok(ResizeRule::Width),
            "height" => Ok(ResizeRule::Height),
            _ => Err(ValueParseError::new(format!("Invalid value '{}'", s))),
        }
    }
}

const HELP_TEXT_RESIZE_ARGS: &str = "
Resize image according to rule and parameters.

Format:
  <rule>:key=value[,key=value...]

Rules:
  no_resize | size | scale | short_edge | long_edge | width | height

Keys:
  edge_size=<pixels>            Set the shorter or longer side (Depends on the rule) of the original image to this value.
                                Required when rule in (short_edge, long_edge).
  ratio=<0-1>                   Scale ratio of output image's width and height.
                                Required when rule is `scale` if width and height is not set.
                                If `ratio` is set, it takes precedence and ignores the settings of `w` and `h`
  w=<px|0-1>                    Pixel or scale ratio of output image's width.
                                Required when rule in (size, width) or when rule is `scale` while `ratio` is not set
  h=<px|0-1>                    Pixel or scale ratio of output image's height.
                                Required when rule in (size, height) or when rule is `scale` while `ratio` is not set
  donot_enlarge=<true|false>    Allow enlarging if the origin image size is smaller then given value (default: false)
  keep_aspect_ratio=<bool>      Keep aspect ratio (default: true).

Examples:
  short_edge:edge_size=300
  size:width=800,height=600
  scale:ratio=0.8
  scale:w=0.8,h=0.7
";
#[derive(Clone, Debug)]
pub struct ResizeArgs {
    pub rule: ResizeRule,
    pub edge_size: u32,
    pub width: f32,
    pub height: f32,
    pub ratio: f32,
    pub donot_enlarge: bool,
    pub keep_aspect_ratio: bool,
}

impl ResizeArgs {
    fn new() -> Self {
        Self {
            rule: ResizeRule::NoResize,
            edge_size: 0,
            width: 0.0,
            height: 0.0,
            ratio: 0.0,
            donot_enlarge: false,
            keep_aspect_ratio: true,
        }
    }
}

impl str::FromStr for ResizeArgs {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.splitn(2, ':').collect::<Vec<&str>>();

        let rule = ResizeRule::from_str(split.get(0).unwrap_or(&s))?;
        let args_map = parse_kv(split.get(1).unwrap_or(&""));

        let edge_size_arg = args_map.get("edge_size");
        let width_arg = args_map.get("w");
        let height_arg = args_map.get("h");
        let ratio_arg = args_map.get("ratio");

        let mut resize_args = ResizeArgs::new();
        resize_args.rule = rule;

        if let Some(enlarge) = args_map.get("donot_enlarge") {
            resize_args.donot_enlarge = enlarge.parse()?;
        }

        if let Some(keep_aspect_ratio) = args_map.get("keep_aspect_ratio") {
            resize_args.keep_aspect_ratio = keep_aspect_ratio.parse()?;
        }

        match rule {
            ResizeRule::Size => {
                if width_arg.is_none() || height_arg.is_none() {
                    return Err(ValueParseError::from(
                        "width and height is required when resize rule is `size`",
                    ));
                }

                resize_args.width = width_arg.unwrap().parse()?;
                resize_args.height = height_arg.unwrap().parse()?;
            }
            ResizeRule::Scale => {
                if ratio_arg.is_some() {
                    resize_args.ratio = ratio_arg.unwrap().parse()?
                } else if width_arg.is_none() || height_arg.is_none() {
                    return Err(ValueParseError::from(
                        "width and height is required when resize rule is `scale` and `ratio` is not set.",
                    ));
                } else {
                    resize_args.width = width_arg.unwrap().parse()?;
                    resize_args.height = height_arg.unwrap().parse()?;
                }
            }
            ResizeRule::ShortEdge | ResizeRule::LongEdge => {
                if edge_size_arg.is_none() {
                    return Err(ValueParseError::from(
                        "edge_size is required when resize rule is `short_edge` or `long_edge`",
                    ));
                }

                resize_args.edge_size = edge_size_arg.unwrap().parse()?;
            }
            ResizeRule::Width => {
                if width_arg.is_none() {
                    return Err(ValueParseError::from(
                        "width is required when resize rule is `width`",
                    ));
                }

                resize_args.width = width_arg.unwrap().parse()?;
            }
            ResizeRule::Height => {
                if height_arg.is_none() {
                    return Err(ValueParseError::from(
                        "height is required when resize rule is `height`",
                    ));
                }

                resize_args.height = height_arg.unwrap().parse()?;
            }
            _ => {}
        }

        Ok(resize_args)
    }
}

// jpeg

#[derive(Clone, Debug)]
pub enum ChromaSubsampling {
    CS444,
    CS422,
    CS420,
    CS411,
    Auto,
}

impl str::FromStr for ChromaSubsampling {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cs444" => Ok(ChromaSubsampling::CS444),
            "cs422" => Ok(ChromaSubsampling::CS422),
            "cs420" => Ok(ChromaSubsampling::CS420),
            "cs411" => Ok(ChromaSubsampling::CS411),
            "auto" => Ok(ChromaSubsampling::Auto),
            _ => Err(ValueParseError::new(format!("Invalid value '{}'", s))),
        }
    }
}

impl From<ChromaSubsampling> for cs_params::ChromaSubsampling {
    fn from(v: ChromaSubsampling) -> cs_params::ChromaSubsampling {
        match v {
            ChromaSubsampling::CS444 => cs_params::ChromaSubsampling::CS444,
            ChromaSubsampling::CS422 => cs_params::ChromaSubsampling::CS422,
            ChromaSubsampling::CS420 => cs_params::ChromaSubsampling::CS420,
            ChromaSubsampling::CS411 => cs_params::ChromaSubsampling::CS411,
            ChromaSubsampling::Auto => cs_params::ChromaSubsampling::Auto,
        }
    }
}

const HELP_TEXT_JPEG_PARAMS: &str = "
JPEG output options: key=value[,key=value...]
Keys:
  quality=<0-100>                Image quality
  chroma_subsampling=<cs444|cs422|cs420|cs411|auto>  Chroma subsampling (default: auto)
  progressive=<true|false>       Use progressive JPEG (default: true)
Example: --jpeg-params quality=80,chroma_subsampling=auto
";
#[derive(Clone, Debug)]
pub struct JpegParams {
    pub quality: u32,
    pub chroma_subsampling: ChromaSubsampling,
    pub progressive: bool,
}

impl str::FromStr for JpegParams {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = parse_kv(s);

        let quality = params.get("quality").unwrap_or(&"80").parse()?;

        let chroma_subsampling = params
            .get("chroma_subsampling")
            .unwrap_or(&"auto")
            .parse()?;

        let progressive = params.get("progressive").unwrap_or(&"true").parse()?;

        Ok(Self {
            quality,
            chroma_subsampling,
            progressive,
        })
    }
}

impl From<JpegParams> for cs_params::JpegParameters {
    fn from(v: JpegParams) -> Self {
        cs_params::JpegParameters {
            quality: v.quality,
            chroma_subsampling: v.chroma_subsampling.into(),
            progressive: v.progressive,
        }
    }
}

// png

const HELP_TEXT_PNG_PARAMS: &str = "
PNG output options: key=value[,key=value...]
Keys:
  quality=<0-100>                Image quality (default: 80)
  force_zopfli=<true|false>      Use Zopfli compression (slow, default: false)
  optimization_level=<0-6>       PNG optimization level (default: 2)
Example: --png-params quality=80,force_zopfli=false
";
#[derive(Clone, Debug)]
pub struct PngParams {
    pub quality: u32,
    pub force_zopfli: bool,
    pub optimization_level: u8,
}

impl str::FromStr for PngParams {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = parse_kv(s);

        let quality = params.get("quality").unwrap_or(&"80").parse()?;
        let force_zopfli = params.get("force_zopfli").unwrap_or(&"false").parse()?;
        let optimization_level = params.get("optimization_level").unwrap_or(&"2").parse()?;

        Ok(Self {
            quality,
            force_zopfli,
            optimization_level,
        })
    }
}

impl From<PngParams> for cs_params::PngParameters {
    fn from(v: PngParams) -> Self {
        cs_params::PngParameters {
            quality: v.quality,
            force_zopfli: v.force_zopfli,
            optimization_level: v.optimization_level,
        }
    }
}

// gif

const HELP_TEXT_GIF_PARAMS: &str = "
GIF output options: key=value[,key=value...]
Keys:
  quality=<0-100>                Image quality (default: 80)
Example: --gif-params quality=80
";
#[derive(Clone, Debug)]
pub struct GifParams {
    pub quality: u32,
}

impl str::FromStr for GifParams {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = parse_kv(s);

        let quality = params.get("quality").unwrap_or(&"80").parse()?;

        Ok(Self { quality })
    }
}

impl From<GifParams> for cs_params::GifParameters {
    fn from(value: GifParams) -> Self {
        cs_params::GifParameters {
            quality: value.quality,
        }
    }
}

// webp

const HELP_TEXT_WEBP_PARAMS: &str = "
WebP output options: key=value[,key=value...]
Keys:
  quality=<0-100>                Image quality (default: 80)
Example: --webp-params quality=80
";
#[derive(Clone, Debug)]
pub struct WebPParams {
    pub quality: u32,
}

impl str::FromStr for WebPParams {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = parse_kv(s);

        let quality = params.get("quality").unwrap_or(&"80").parse()?;

        Ok(Self { quality })
    }
}

impl From<WebPParams> for cs_params::WebPParameters {
    fn from(v: WebPParams) -> Self {
        cs_params::WebPParameters { quality: v.quality }
    }
}

// tiff

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum TiffCompression {
    Uncompressed = 0,
    Lzw = 1,
    #[default]
    Deflate = 2,
    Packbits = 3,
}

impl str::FromStr for TiffCompression {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "uncompressed" => Ok(Self::Uncompressed),
            "lzw" => Ok(Self::Lzw),
            "deflate" => Ok(Self::Deflate),
            "packbits" => Ok(Self::Packbits),
            _ => Err(ValueParseError::new(format!("Invalid value '{}'", s))),
        }
    }
}

impl From<TiffCompression> for cs_params::TiffCompression {
    fn from(v: TiffCompression) -> Self {
        match v {
            TiffCompression::Uncompressed => cs_params::TiffCompression::Uncompressed,
            TiffCompression::Lzw => cs_params::TiffCompression::Lzw,
            TiffCompression::Deflate => cs_params::TiffCompression::Deflate,
            TiffCompression::Packbits => cs_params::TiffCompression::Packbits,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum TiffDeflateLevel {
    Fast = 1,
    #[default]
    Balanced = 6,
    Best = 9,
}

impl str::FromStr for TiffDeflateLevel {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "fast" => Ok(Self::Fast),
            "balanced" => Ok(Self::Balanced),
            "best" => Ok(Self::Best),
            _ => Err(ValueParseError::new(format!("Invalid value '{}'", s))),
        }
    }
}

impl From<TiffDeflateLevel> for cs_params::TiffDeflateLevel {
    fn from(v: TiffDeflateLevel) -> Self {
        match v {
            TiffDeflateLevel::Fast => cs_params::TiffDeflateLevel::Fast,
            TiffDeflateLevel::Balanced => cs_params::TiffDeflateLevel::Balanced,
            TiffDeflateLevel::Best => cs_params::TiffDeflateLevel::Best,
        }
    }
}

const HELP_TEXT_TIFF_PARAMS: &str = "
TIFF output options: key=value[,key=value...]
Keys:
  algorithm=<uncompressed|lzw|deflate|packbits>   Compression algorithm (default: deflate)
  deflate_level=<fast|balanced|best>              Deflate compression level (default: balanced)
Example: --tiff-params algorithm=deflate,deflate_level=balanced
";
#[derive(Clone, Debug)]
pub struct TiffParams {
    pub algorithm: TiffCompression,
    pub deflate_level: TiffDeflateLevel,
}

impl str::FromStr for TiffParams {
    type Err = ValueParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = parse_kv(s);

        let algorithm = params.get("algorithm").unwrap_or(&"deflate").parse()?;
        let deflate_level = params.get("deflate_level").unwrap_or(&"balanced").parse()?;

        Ok(Self {
            algorithm,
            deflate_level,
        })
    }
}

impl From<TiffParams> for cs_params::TiffParameters {
    fn from(v: TiffParams) -> Self {
        cs_params::TiffParameters {
            algorithm: v.algorithm.into(),
            deflate_level: v.deflate_level.into(),
        }
    }
}

// cli options

#[derive(StructOpt, Clone, Debug)]
#[structopt(
    name = "imgtool",
    about = "A simple tool to compress and convert images based on libcaesium."
)]
pub struct CliOptions {
    /// Input file or directory
    #[structopt(short, long)]
    pub input: PathBuf,

    /// Output file or directory.
    #[structopt(short, long)]
    pub output: PathBuf,

    /// Prefix of output file, if not set, the filename of the output file is same to the origin file.
    #[structopt(short, long)]
    pub prefix: Option<String>,

    /// Suffix of output file, if not set, the filename of the output file is same to the origin file.
    #[structopt(short, long)]
    pub suffix: Option<String>,

    /// Just print the processing plan, no output file
    #[structopt(long)]
    pub dry_run: bool,

    /// Whether to continue with the remaining tasks when an error occurs while processing a file
    #[structopt(long)]
    pub continue_on_error: bool,

    /// If the output file size is bigger than the source file, skip (do not output)
    #[structopt(long)]
    pub skip_if_bigger: bool,

    /// Output format, if not set, keep same to the original image.
    /// Available values: [jpg, jpeg, png, gif, webp, tiff]
    #[structopt(short, long)]
    pub target_format: Option<OutputFormatTypes>,

    /// Whether to delete the origin file after process finish.
    #[structopt(long)]
    pub delete_origin: bool,

    /// Whether to keep metadata in the compressed image
    #[structopt(long)]
    pub keep_metadata: bool,

    /// Whether to use lossless compression (quality may still decline)
    #[structopt(long)]
    pub lossless: bool,

    #[structopt(long, default_value = "no_resize", help = HELP_TEXT_RESIZE_ARGS)]
    pub resize_args: ResizeArgs,

    #[structopt(long, help = HELP_TEXT_JPEG_PARAMS)]
    pub jpeg_params: Option<JpegParams>,

    #[structopt(long, help = HELP_TEXT_PNG_PARAMS)]
    pub png_params: Option<PngParams>,

    #[structopt(long, help = HELP_TEXT_GIF_PARAMS)]
    pub gif_params: Option<GifParams>,

    #[structopt(long, help = HELP_TEXT_WEBP_PARAMS)]
    pub webp_params: Option<WebPParams>,

    #[structopt(long, help = HELP_TEXT_TIFF_PARAMS)]
    pub tiff_params: Option<TiffParams>,
}

impl From<CliOptions> for cs_params::CSParameters {
    fn from(cli_opt: CliOptions) -> Self {
        let mut cs_params = cs_params::CSParameters::new();

        if let Some(params) = cli_opt.jpeg_params {
            cs_params.jpeg = params.into();
        }
        if let Some(params) = cli_opt.png_params {
            cs_params.png = params.into();
        }
        if let Some(params) = cli_opt.gif_params {
            cs_params.gif = params.into();
        }
        if let Some(params) = cli_opt.webp_params {
            cs_params.webp = params.into();
        }
        if let Some(params) = cli_opt.tiff_params {
            cs_params.tiff = params.into();
        }

        cs_params.keep_metadata = cli_opt.keep_metadata;
        cs_params.optimize = cli_opt.lossless;

        cs_params
    }
}
