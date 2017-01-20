use decoders::*;
use decoders::tiff::*;
use decoders::ljpeg::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct TfrDecoder<'a> {
  buffer: &'a [u8],
  rawloader: &'a RawLoader,
  tiff: TiffIFD<'a>,
}

impl<'a> TfrDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawloader: &'a RawLoader) -> TfrDecoder<'a> {
    TfrDecoder {
      buffer: buf,
      tiff: tiff,
      rawloader: rawloader,
    }
  }
}

impl<'a> Decoder for TfrDecoder<'a> {
  fn image(&self) -> Result<RawImage,String> {
    let camera = try!(self.rawloader.check_supported(&self.tiff));
    let raw = fetch_ifd!(&self.tiff, Tag::WhiteLevel);
    let width = fetch_tag!(raw, Tag::ImageWidth).get_usize(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_usize(0);
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_usize(0);
    let src = &self.buffer[offset..];

    let image = try!(self.decode_compressed(src, width, height));
    ok_image(camera, width, height, try!(self.get_wb()), image)
  }
}

impl<'a> TfrDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let levels = fetch_tag!(self.tiff, Tag::AsShotNeutral);
    Ok([1.0/levels.get_f32(0),1.0/levels.get_f32(1),1.0/levels.get_f32(2),NAN])
  }

  fn decode_compressed(&self, src: &[u8], width: usize, height: usize) -> Result<Vec<u16>,String> {
    let mut out = vec![0 as u16; width*height];
    let decompressor = try!(LjpegDecompressor::new_full(src, true, false));
    try!(decompressor.decode(&mut out, 0, width, width, height));
    Ok(out)
  }
}
