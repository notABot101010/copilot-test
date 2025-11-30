// API functions for interacting with the S3 server

export interface Bucket {
  name: string;
  creationDate: string;
}

export interface S3Object {
  key: string;
  lastModified: string;
  etag: string;
  size: number;
  storageClass: string;
}

export interface ListObjectsResult {
  name: string;
  prefix: string;
  keyCount: number;
  maxKeys: number;
  isTruncated: boolean;
  contents: S3Object[];
  commonPrefixes: string[];
  nextContinuationToken?: string;
}

const API_BASE = '/api';

// Parse S3 XML response for buckets
function parseListBucketsXml(xml: string): Bucket[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, 'application/xml');
  const buckets: Bucket[] = [];

  const bucketNodes = doc.querySelectorAll('Bucket');
  bucketNodes.forEach((node) => {
    const name = node.querySelector('Name')?.textContent || '';
    const creationDate = node.querySelector('CreationDate')?.textContent || '';
    buckets.push({ name, creationDate });
  });

  return buckets;
}

// Parse S3 XML response for objects
function parseListObjectsXml(xml: string): ListObjectsResult {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, 'application/xml');

  const result: ListObjectsResult = {
    name: doc.querySelector('Name')?.textContent || '',
    prefix: doc.querySelector('Prefix')?.textContent || '',
    keyCount: parseInt(doc.querySelector('KeyCount')?.textContent || '0'),
    maxKeys: parseInt(doc.querySelector('MaxKeys')?.textContent || '1000'),
    isTruncated: doc.querySelector('IsTruncated')?.textContent === 'true',
    contents: [],
    commonPrefixes: [],
  };

  const nextToken = doc.querySelector('NextContinuationToken')?.textContent;
  if (nextToken) {
    result.nextContinuationToken = nextToken;
  }

  const contentNodes = doc.querySelectorAll('Contents');
  contentNodes.forEach((node) => {
    const obj: S3Object = {
      key: node.querySelector('Key')?.textContent || '',
      lastModified: node.querySelector('LastModified')?.textContent || '',
      etag: node.querySelector('ETag')?.textContent || '',
      size: parseInt(node.querySelector('Size')?.textContent || '0'),
      storageClass: node.querySelector('StorageClass')?.textContent || 'STANDARD',
    };
    result.contents.push(obj);
  });

  const prefixNodes = doc.querySelectorAll('CommonPrefixes Prefix');
  prefixNodes.forEach((node) => {
    if (node.textContent) {
      result.commonPrefixes.push(node.textContent);
    }
  });

  return result;
}

export async function listBuckets(): Promise<Bucket[]> {
  const response = await fetch(API_BASE);
  if (!response.ok) {
    throw new Error(`Failed to list buckets: ${response.statusText}`);
  }
  const xml = await response.text();
  return parseListBucketsXml(xml);
}

export async function createBucket(name: string): Promise<void> {
  const response = await fetch(`${API_BASE}/${name}`, {
    method: 'PUT',
  });
  if (!response.ok) {
    throw new Error(`Failed to create bucket: ${response.statusText}`);
  }
}

export async function deleteBucket(name: string): Promise<void> {
  const response = await fetch(`${API_BASE}/${name}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error(`Failed to delete bucket: ${response.statusText}`);
  }
}

export async function listObjects(
  bucket: string,
  prefix?: string,
  delimiter?: string,
  maxKeys?: number,
  continuationToken?: string
): Promise<ListObjectsResult> {
  const params = new URLSearchParams();
  if (prefix) params.set('prefix', prefix);
  if (delimiter) params.set('delimiter', delimiter);
  if (maxKeys) params.set('max-keys', maxKeys.toString());
  if (continuationToken) params.set('continuation-token', continuationToken);

  const url = `${API_BASE}/${bucket}${params.toString() ? '?' + params.toString() : ''}`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to list objects: ${response.statusText}`);
  }
  const xml = await response.text();
  return parseListObjectsXml(xml);
}

export async function uploadObject(
  bucket: string,
  key: string,
  file: File,
  _onProgress?: (percent: number) => void
): Promise<void> {
  const response = await fetch(`${API_BASE}/${bucket}/${key}`, {
    method: 'PUT',
    headers: {
      'Content-Type': file.type || 'application/octet-stream',
    },
    body: file,
  });

  if (!response.ok) {
    throw new Error(`Failed to upload object: ${response.statusText}`);
  }
}

export async function deleteObject(bucket: string, key: string): Promise<void> {
  const response = await fetch(`${API_BASE}/${bucket}/${key}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error(`Failed to delete object: ${response.statusText}`);
  }
}

export async function getObjectUrl(bucket: string, key: string): Promise<string> {
  return `${API_BASE}/${bucket}/${key}`;
}

export async function headObject(bucket: string, key: string): Promise<{ size: number; etag: string; contentType?: string; lastModified: string }> {
  const response = await fetch(`${API_BASE}/${bucket}/${key}`, {
    method: 'HEAD',
  });
  if (!response.ok) {
    throw new Error(`Failed to head object: ${response.statusText}`);
  }

  return {
    size: parseInt(response.headers.get('content-length') || '0'),
    etag: response.headers.get('etag') || '',
    contentType: response.headers.get('content-type') || undefined,
    lastModified: response.headers.get('last-modified') || '',
  };
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString();
  } catch {
    return dateStr;
  }
}
