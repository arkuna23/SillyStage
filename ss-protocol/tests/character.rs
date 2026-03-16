use std::io::Write;

use ss_protocol::{
    CHARACTER_ARCHIVE_CONTENT_PATH, CHARACTER_ARCHIVE_MANIFEST_PATH, CharacterArchive,
    CharacterArchiveError, CharacterArchiveManifest, CharacterCardContent, CharacterCoverMimeType,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn sample_content() -> CharacterCardContent {
    CharacterCardContent {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        schema_id: "schema-character-merchant".to_owned(),
        system_prompt: "Stay in character.".to_owned(),
    }
}

#[test]
fn chr_archive_round_trip_preserves_manifest_content_and_cover() {
    let archive = CharacterArchive::new(
        sample_content(),
        CharacterCoverMimeType::Png,
        b"fake-cover".to_vec(),
    );

    let bytes = archive.to_chr_bytes().expect("archive should serialize");
    let parsed = CharacterArchive::from_chr_bytes(&bytes).expect("archive should deserialize");
    let summary = parsed.summary();

    assert_eq!(summary.character_id, "merchant");
    assert_eq!(summary.cover_file_name.as_deref(), Some("cover.png"));
    assert_eq!(summary.cover_mime_type, Some(CharacterCoverMimeType::Png));

    let reader = std::io::Cursor::new(bytes);
    let mut zip = ZipArchive::new(reader).expect("zip should open");
    assert!(zip.by_name(CHARACTER_ARCHIVE_MANIFEST_PATH).is_ok());
    assert!(zip.by_name(CHARACTER_ARCHIVE_CONTENT_PATH).is_ok());
    assert!(zip.by_name("cover.png").is_ok());
}

#[test]
fn chr_archive_rejects_mismatched_character_ids() {
    let archive = CharacterArchive {
        manifest: CharacterArchiveManifest::new(
            "other_merchant",
            CharacterCoverMimeType::Jpeg,
            "cover.jpg",
        ),
        content: sample_content(),
        cover_bytes: b"fake-cover".to_vec(),
    };

    let error = archive
        .to_chr_bytes()
        .expect_err("mismatched ids should fail");
    assert!(matches!(
        error,
        CharacterArchiveError::CharacterIdMismatch { .. }
    ));
}

#[test]
fn chr_archive_rejects_missing_cover_entry() {
    let mut bytes = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(&mut bytes);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let manifest =
        CharacterArchiveManifest::new("merchant", CharacterCoverMimeType::Webp, "cover.webp");
    writer
        .start_file(CHARACTER_ARCHIVE_MANIFEST_PATH, options)
        .expect("manifest entry should start");
    writer
        .write_all(&serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"))
        .expect("manifest should write");
    writer
        .start_file(CHARACTER_ARCHIVE_CONTENT_PATH, options)
        .expect("content entry should start");
    writer
        .write_all(&serde_json::to_vec_pretty(&sample_content()).expect("content should serialize"))
        .expect("content should write");
    writer.finish().expect("zip should finish");

    let error = CharacterArchive::from_chr_bytes(&bytes.into_inner())
        .expect_err("missing cover entry should fail");
    assert!(matches!(
        error,
        CharacterArchiveError::MissingArchiveEntry(name) if name == "cover.webp"
    ));
}
