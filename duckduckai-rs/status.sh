#!/bin/bash

# Make request to DuckDuckGo status endpoint
response=$(curl -i -s "https://duckduckgo.com/duckchat/v1/status" \
  -H "authority: duckduckgo.com" \
  -H "method: GET" \
  -H "path: /duckchat/v1/status" \
  -H "scheme: https" \
  -H "accept: */*" \
  -H "accept-encoding: gzip, deflate, br, zstd" \
  -H "accept-language: fr-FR,fr;q=0.6" \
  -H "cache-control: no-store" \
  -H "dnt: 1" \
  -H "priority: u=1, i" \
  -H "referer: https://duckduckgo.com/" \
  -H 'sec-ch-ua: "Not)A;Brand";v="8", "Chromium";v="138", "Brave";v="138"' \
  -H "sec-ch-ua-mobile: ?0" \
  -H 'sec-ch-ua-platform: "Windows"' \
  -H "sec-fetch-dest: empty" \
  -H "sec-fetch-mode: cors" \
  -H "sec-fetch-site: same-origin" \
  -H "sec-gpc: 1" \
  -H "x-vqd-accept: 1" \
  -H "user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36" \
  -b "5=1; dcs=1; dcm=3" \
  --compressed)

# Extract status code
status_code=$(echo "$response" | grep -i "^HTTP" | tail -1 | awk '{print $2}')

# Extract x-vqd-hash-1 header
vqd_header=$(echo "$response" | grep -i "^x-vqd-hash-1:" | cut -d' ' -f2- | tr -d '\r')

# Extract body (JSON response)
body=$(echo "$response" | sed -n '/^{/,$p')

# Display results
echo "Status Code: $status_code"
echo ""
echo "VQD Header (x-vqd-hash-1):"
echo "$vqd_header"
echo ""
echo "Response Body:"
echo "$body" | jq '.' 2>/dev/null || echo "$body"
# echo ""
# echo "All Response Headers:"
# echo "$response" | sed -n '/^HTTP/,/^$/p'
