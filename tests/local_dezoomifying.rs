use dezoomify_rs::{Arguments, dezoomify, ZoomError};
use std::default::Default;
use image::{self, DynamicImage, GenericImageView};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::{Path, PathBuf};

/// Dezoom a file locally
#[ignore] // Ignore this test by default because it's slow in debug mode
#[tokio::test(threaded_scheduler)]
pub async fn custom_size_local_zoomify_tiles() {
    test_image(
        "testdata/zoomify/test_custom_size/ImageProperties.xml",
        "testdata/zoomify/test_custom_size/expected_result.jpg",
    ).await.unwrap()
}

#[tokio::test(threaded_scheduler)]
pub async fn local_generic_tiles() {
    test_image(
        "testdata/generic/map_{{X}}_{{Y}}.jpg",
        "testdata/generic/map_expected.jpg",
    ).await.unwrap()
}

pub async fn dezoom_image<'a>(input: &str, expected: &'a str) -> Result<TmpFile<'a>, ZoomError> {
    let mut args: Arguments = Default::default();
    args.input_uri = Some(input.into());
    args.largest = true;
    args.retries = 0;
    args.logging = "error".into();

    let tmp_file = TmpFile(expected);
    args.outfile = Some(tmp_file.to_path_buf());
    dezoomify(&args).await.expect("Dezooming failed");
    Ok(tmp_file)
}

pub async fn test_image(input: &str, expected: &str) -> Result<(), ZoomError> {
    let tmp_file = dezoom_image(input, expected).await?;
    let actual = image::open(tmp_file.to_path_buf())?;
    let expected = image::open(expected)?;
    assert_images_equal(actual, expected);
    Ok(())
}

fn assert_images_equal(a: DynamicImage, b: DynamicImage) {
    assert_eq!(a.dimensions(), b.dimensions(), "image dimensions should match");
    for ((x, y, a), (_, _, b)) in a.pixels().zip(b.pixels()) {
        for (&pa, &pb) in a.0.iter().zip(b.0.iter()) {
            assert!(pa.max(pb) - pa.min(pb) < 20,
                    "The pixels differ in ({}, {}): {:?} !~= {:?}", x, y, a, b
            );
        }
    }
}

pub struct TmpFile<'a>(&'a str);

impl<'a> TmpFile<'a> {
    fn to_path_buf(&'a self) -> PathBuf {
        let mut out_file = std::env::temp_dir();
        out_file.push(format!("dezoomify-out-{}", hash(self.0)));
        let orig_path: &Path = self.0.as_ref();
        out_file.set_extension(orig_path.extension().expect("missing extension"));
        out_file
    }
}

impl<'a> Drop for TmpFile<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.to_path_buf());
    }
}

fn hash<T: Hash>(v: T) -> u64 {
    let mut s = DefaultHasher::new();
    v.hash(&mut s);
    s.finish()
}