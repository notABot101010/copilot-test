//! XML response builders for S3 API

use crate::database::{Bucket, MultipartUpload, ObjectMetadata, UploadPart};
use serde::Serialize;

/// Error response structure
#[derive(Debug, Serialize)]
#[serde(rename = "Error")]
pub struct ErrorResponse {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Resource")]
    pub resource: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str, resource: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            resource: resource.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>{}</Code>
    <Message>{}</Message>
    <Resource>{}</Resource>
    <RequestId>{}</RequestId>
</Error>"#,
            self.code, self.message, self.resource, self.request_id
        )
    }
}

/// List buckets response
#[derive(Debug, Serialize)]
#[serde(rename = "ListAllMyBucketsResult")]
pub struct ListBucketsResponse {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "Buckets")]
    pub buckets: BucketsWrapper,
}

#[derive(Debug, Serialize)]
pub struct BucketsWrapper {
    #[serde(rename = "Bucket")]
    pub bucket: Vec<BucketInfo>,
}

#[derive(Debug, Serialize)]
pub struct Owner {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct BucketInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
}

impl ListBucketsResponse {
    pub fn new(buckets: Vec<Bucket>, owner_id: &str, owner_name: &str) -> Self {
        Self {
            xmlns: "http://s3.amazonaws.com/doc/2006-03-01/".to_string(),
            owner: Owner {
                id: owner_id.to_string(),
                display_name: owner_name.to_string(),
            },
            buckets: BucketsWrapper {
                bucket: buckets
                    .into_iter()
                    .map(|b| BucketInfo {
                        name: b.name,
                        creation_date: format_s3_date(&b.created_at),
                    })
                    .collect(),
            },
        }
    }

    pub fn to_xml(&self) -> String {
        let buckets_xml: String = self
            .buckets
            .bucket
            .iter()
            .map(|b| {
                format!(
                    "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
                    b.name, b.creation_date
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Owner>
        <ID>{}</ID>
        <DisplayName>{}</DisplayName>
    </Owner>
    <Buckets>{}</Buckets>
</ListAllMyBucketsResult>"#,
            self.owner.id, self.owner.display_name, buckets_xml
        )
    }
}

/// List objects response
#[derive(Debug)]
pub struct ListObjectsV2Response {
    pub name: String,
    pub prefix: String,
    pub key_count: i32,
    pub max_keys: i32,
    pub is_truncated: bool,
    pub contents: Vec<ObjectInfo>,
    pub common_prefixes: Vec<String>,
    pub continuation_token: Option<String>,
    pub next_continuation_token: Option<String>,
}

#[derive(Debug)]
pub struct ObjectInfo {
    pub key: String,
    pub last_modified: String,
    pub etag: String,
    pub size: i64,
    pub storage_class: String,
}

impl ListObjectsV2Response {
    pub fn new(
        bucket_name: &str,
        prefix: &str,
        objects: Vec<ObjectMetadata>,
        common_prefixes: Vec<String>,
        max_keys: i32,
        continuation_token: Option<String>,
        next_continuation_token: Option<String>,
    ) -> Self {
        let is_truncated = next_continuation_token.is_some();
        let key_count = objects.len() as i32;

        Self {
            name: bucket_name.to_string(),
            prefix: prefix.to_string(),
            key_count,
            max_keys,
            is_truncated,
            contents: objects
                .into_iter()
                .map(|o| ObjectInfo {
                    key: o.key,
                    last_modified: format_s3_date(&o.last_modified),
                    etag: o.etag,
                    size: o.size,
                    storage_class: "STANDARD".to_string(),
                })
                .collect(),
            common_prefixes,
            continuation_token,
            next_continuation_token,
        }
    }

    pub fn to_xml(&self) -> String {
        let contents_xml: String = self
            .contents
            .iter()
            .map(|o| {
                format!(
                    r#"<Contents>
        <Key>{}</Key>
        <LastModified>{}</LastModified>
        <ETag>{}</ETag>
        <Size>{}</Size>
        <StorageClass>{}</StorageClass>
    </Contents>"#,
                    xml_escape(&o.key),
                    o.last_modified,
                    o.etag,
                    o.size,
                    o.storage_class
                )
            })
            .collect();

        let prefixes_xml: String = self
            .common_prefixes
            .iter()
            .map(|p| format!("<CommonPrefixes><Prefix>{}</Prefix></CommonPrefixes>", xml_escape(p)))
            .collect();

        let continuation_xml = self
            .continuation_token
            .as_ref()
            .map(|t| format!("<ContinuationToken>{}</ContinuationToken>", t))
            .unwrap_or_default();

        let next_continuation_xml = self
            .next_continuation_token
            .as_ref()
            .map(|t| format!("<NextContinuationToken>{}</NextContinuationToken>", t))
            .unwrap_or_default();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Name>{}</Name>
    <Prefix>{}</Prefix>
    <KeyCount>{}</KeyCount>
    <MaxKeys>{}</MaxKeys>
    <IsTruncated>{}</IsTruncated>
    {}
    {}
    {}
    {}
</ListBucketResult>"#,
            self.name,
            xml_escape(&self.prefix),
            self.key_count,
            self.max_keys,
            self.is_truncated,
            continuation_xml,
            next_continuation_xml,
            contents_xml,
            prefixes_xml
        )
    }
}

/// Initiate multipart upload response
#[derive(Debug)]
pub struct InitiateMultipartUploadResponse {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

impl InitiateMultipartUploadResponse {
    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Bucket>{}</Bucket>
    <Key>{}</Key>
    <UploadId>{}</UploadId>
</InitiateMultipartUploadResult>"#,
            self.bucket,
            xml_escape(&self.key),
            self.upload_id
        )
    }
}

/// Complete multipart upload response
#[derive(Debug)]
pub struct CompleteMultipartUploadResponse {
    pub location: String,
    pub bucket: String,
    pub key: String,
    pub etag: String,
}

impl CompleteMultipartUploadResponse {
    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Location>{}</Location>
    <Bucket>{}</Bucket>
    <Key>{}</Key>
    <ETag>{}</ETag>
</CompleteMultipartUploadResult>"#,
            self.location,
            self.bucket,
            xml_escape(&self.key),
            self.etag
        )
    }
}

/// List multipart uploads response
#[derive(Debug)]
pub struct ListMultipartUploadsResponse {
    pub bucket: String,
    pub uploads: Vec<UploadInfo>,
}

#[derive(Debug)]
pub struct UploadInfo {
    pub key: String,
    pub upload_id: String,
    pub initiated: String,
}

impl ListMultipartUploadsResponse {
    pub fn new(bucket: &str, uploads: Vec<MultipartUpload>) -> Self {
        Self {
            bucket: bucket.to_string(),
            uploads: uploads
                .into_iter()
                .map(|u| UploadInfo {
                    key: u.key,
                    upload_id: u.upload_id,
                    initiated: format_s3_date(&u.created_at),
                })
                .collect(),
        }
    }

    pub fn to_xml(&self) -> String {
        let uploads_xml: String = self
            .uploads
            .iter()
            .map(|u| {
                format!(
                    r#"<Upload>
        <Key>{}</Key>
        <UploadId>{}</UploadId>
        <Initiated>{}</Initiated>
    </Upload>"#,
                    xml_escape(&u.key),
                    u.upload_id,
                    u.initiated
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListMultipartUploadsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Bucket>{}</Bucket>
    {}
</ListMultipartUploadsResult>"#,
            self.bucket, uploads_xml
        )
    }
}

/// List parts response
#[derive(Debug)]
pub struct ListPartsResponse {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<PartInfo>,
}

#[derive(Debug)]
pub struct PartInfo {
    pub part_number: i32,
    pub etag: String,
    pub size: i64,
    pub last_modified: String,
}

impl ListPartsResponse {
    pub fn new(bucket: &str, key: &str, upload_id: &str, parts: Vec<UploadPart>) -> Self {
        Self {
            bucket: bucket.to_string(),
            key: key.to_string(),
            upload_id: upload_id.to_string(),
            parts: parts
                .into_iter()
                .map(|p| PartInfo {
                    part_number: p.part_number,
                    etag: p.etag,
                    size: p.size,
                    last_modified: format_s3_date(&p.created_at),
                })
                .collect(),
        }
    }

    pub fn to_xml(&self) -> String {
        let parts_xml: String = self
            .parts
            .iter()
            .map(|p| {
                format!(
                    r#"<Part>
        <PartNumber>{}</PartNumber>
        <ETag>{}</ETag>
        <Size>{}</Size>
        <LastModified>{}</LastModified>
    </Part>"#,
                    p.part_number, p.etag, p.size, p.last_modified
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Bucket>{}</Bucket>
    <Key>{}</Key>
    <UploadId>{}</UploadId>
    {}
</ListPartsResult>"#,
            self.bucket,
            xml_escape(&self.key),
            self.upload_id,
            parts_xml
        )
    }
}

/// Copy object response
#[derive(Debug)]
pub struct CopyObjectResponse {
    pub etag: String,
    pub last_modified: String,
}

impl CopyObjectResponse {
    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CopyObjectResult>
    <ETag>{}</ETag>
    <LastModified>{}</LastModified>
</CopyObjectResult>"#,
            self.etag, self.last_modified
        )
    }
}

/// Delete objects response
#[derive(Debug)]
pub struct DeleteObjectsResponse {
    pub deleted: Vec<DeletedObject>,
    pub errors: Vec<DeleteError>,
}

#[derive(Debug)]
pub struct DeletedObject {
    pub key: String,
}

#[derive(Debug)]
pub struct DeleteError {
    pub key: String,
    pub code: String,
    pub message: String,
}

impl DeleteObjectsResponse {
    pub fn to_xml(&self) -> String {
        let deleted_xml: String = self
            .deleted
            .iter()
            .map(|d| format!("<Deleted><Key>{}</Key></Deleted>", xml_escape(&d.key)))
            .collect();

        let errors_xml: String = self
            .errors
            .iter()
            .map(|e| {
                format!(
                    "<Error><Key>{}</Key><Code>{}</Code><Message>{}</Message></Error>",
                    xml_escape(&e.key),
                    e.code,
                    e.message
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<DeleteResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    {}
    {}
</DeleteResult>"#,
            deleted_xml, errors_xml
        )
    }
}

/// Format a date string to S3 format (ISO 8601)
fn format_s3_date(date_str: &str) -> String {
    // Assuming date is stored as "YYYY-MM-DD HH:MM:SS" or similar
    // Convert to ISO 8601 format: "YYYY-MM-DDTHH:MM:SS.000Z"
    if date_str.contains('T') {
        return date_str.to_string();
    }

    date_str.replace(' ', "T") + ".000Z"
}

/// Escape special XML characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a<b>c"), "a&lt;b&gt;c");
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_error_response() {
        let err = ErrorResponse::new("NoSuchBucket", "The specified bucket does not exist", "/test-bucket");
        let xml = err.to_xml();
        assert!(xml.contains("<Code>NoSuchBucket</Code>"));
        assert!(xml.contains("<Message>The specified bucket does not exist</Message>"));
    }
}
