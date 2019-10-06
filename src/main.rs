use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use image::io::Reader;
use image::DynamicImage;
use imageproc::stats::{histogram, ChannelHistogram};
use iron::prelude::*;
use iron::status;
use iron_json_response::{JsonResponse, JsonResponseMiddleware};
use multipart::server::{Multipart, MultipartData, MultipartField, ReadEntryResult, SaveResult};
use router::Router;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use tempfile::tempfile_in;

struct Histogram(ChannelHistogram);

struct HistogramChannel([u32; 256]);

impl Serialize for HistogramChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(256))?;
        for i in 0..256 {
            seq.serialize_element(&self.0[i])?;
        }
        seq.end()
    }
}

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.channels.len()))?;
        for (i, e) in (0..).zip(self.0.channels.iter()) {
            map.serialize_entry(&i, &HistogramChannel(*e))?;
        }
        map.end()
    }
}

fn get_first_entry<R: Read>(multipart: Multipart<R>) -> Option<MultipartField<Multipart<R>>> {
    match multipart.into_entry() {
        ReadEntryResult::Entry(entry) => Some(entry),
        ReadEntryResult::End(_) | ReadEntryResult::Error(_, _) => None,
    }
}

fn save_multipart_to_file<R: Read>(
    data: &mut MultipartData<Multipart<R>>,
    file: &mut File,
) -> Option<()> {
    match data.save().size_limit(256 * 1024 * 1024).write_to(file) {
        SaveResult::Full(_) => Some(()),
        SaveResult::Partial(_, _) | SaveResult::Error(_) => None,
    }
}

// We don't care much about errors, so let's just return None whenever we meet
// one. A better implementation would define its own error type.
fn get_histogram_from_request(req: &mut Request) -> Option<Histogram> {
    let multipart = Multipart::from_request(req).ok()?;

    let mut entry = get_first_entry(multipart)?;

    if &*entry.headers.name != "image" {
        return None;
    }

    // We might not have the necessary permission to write in the tmp folder,
    // for instance in a scratch image.
    let mut file = tempfile_in("./").ok()?;

    save_multipart_to_file(&mut entry.data, &mut file);

    // We're writing to a temporary file instead of piping it directly to the
    // image decoder because we need to support seeking.
    file.seek(SeekFrom::Start(0)).ok()?;

    let buf_file = BufReader::new(file);

    // As per the spec, we only have access to the contents of the file, from
    // which we must guess its format.
    let img = Reader::new(buf_file)
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;

    // Rust doesn't feature specialization, so we have to write this out
    // manually. See the description of imageproc's Image type.
    Some(Histogram(match &img {
        DynamicImage::ImageLuma8(buf) => histogram(buf),
        DynamicImage::ImageLumaA8(buf) => histogram(buf),
        DynamicImage::ImageRgb8(buf) => histogram(buf),
        DynamicImage::ImageRgba8(buf) => histogram(buf),
        DynamicImage::ImageBgr8(buf) => histogram(buf),
        DynamicImage::ImageBgra8(buf) => histogram(buf),
    }))
}

fn handler(req: &mut Request) -> IronResult<Response> {
    Ok(match get_histogram_from_request(req) {
        Some(histogram) => {
            let mut resp = Response::new();
            resp.set_mut(JsonResponse::json(histogram))
                .set_mut(status::Ok);
            resp
        }
        None => Response::with((status::BadRequest, "Bad request")),
    })
}

fn main() {
    let mut router = Router::new();
    router.post("/histogram", handler, "histogram");

    let mut chain = Chain::new(router);
    chain.link_after(JsonResponseMiddleware::new());

    Iron::new(chain).http("0.0.0.0:80").unwrap();
}
