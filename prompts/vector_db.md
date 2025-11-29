Create a database to offer hybrid search (full text search and vector search), in Rust.

It should be stateless and store the data on S3, in a pure "serverless" fashion.

It should be accessible via an API over HTTP (use the axum crate), it should have endpoints to write documents and query. Add CORS.

query should support fulltext queries, vector queries and hybrid

documents belong to a namespace.

documents IDs should be string to facilitate usage.

Add 2 layers of caching to not have to query S3 everytime, which is slow: memory and disk.


take inspiration from https://turbopuffer.com/blog/turbopuffer



add both unit and integrations tests


Example of usage
# Basic vector search example
curl http://localhost:8080/api/namespaces/vector-1-example-curl \
  -X POST --fail-with-body \
  -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
   "documents": [
     {"id": "1", "vector": [0.1, 0.2], "text": "A cat sleeping on a windowsill", "category": "animal"},
     {"id": "2", "vector": [0.15, 0.25], "text": "A playful kitten chasing a toy", "category": "animal"},
     {"id": "3", "vector": [0.8, 0.9], "text": "An airplane flying through clouds", "category": "vehicle"}
   ],
   "distance_metric": "cosine_distance"
 }'

curl http://localhost:8080/api/namespaces/vector-1-example-curl/query \
  -X POST --fail-with-body \
  -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
   "rank_by": ["vector", "ANN", [0.12, 0.22]],
   "top_k": 2,
   "include_attributes": ["text"]
 }'
# Returns cat and kitten documents, sorted by vector similarity

# Example of vector search with filters
curl http://localhost:8080/api/namespaces/vector-2-example-curl \
  -X POST --fail-with-body \
  -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
   "documents": [
     {"id": "1", "vector": [0.1, 0.2], "description": "A shiny red sports car", "color": "red", "type": "car", "price": 50000},
     {"id": "2", "vector": [0.15, 0.25], "description": "A sleek blue sedan", "color": "blue", "type": "car", "price": 35000},
     {"id": "3", "vector": [0.3, 0.4], "description": "A large red delivery truck", "color": "red", "type": "truck", "price": 80000},
     {"id": "4", "vector": [0.35, 0.45], "description": "A blue pickup truck", "color": "blue", "type": "truck", "price": 45000}
   ],
   "distance_metric": "cosine_distance"
 }'

curl http://localhost:8080/api/namespaces/vector-2-example-curl/query \
  -X POST --fail-with-body \
  -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
   "rank_by": ["vector", "ANN", [0.12, 0.22]],
   "top_k": 10,
   "filters": ["And", [
     ["price", "Lt", 60000],
     ["color", "Eq", "blue"]
   ]],
   "include_attributes": ["description", "price"]
 }'


use SQLite (and the sqlx crate) to store API keys and any other state.

Then add a simple dashboard with preact with pages for:
- list namespace
- show namespace with basic stats such as the number of documents and query the namespace
- list and explore documents

We should be able to update the BASE_URL env var for the webapp for testing e.g. when we want to use the server which is listenning on another port e.g. http://localhost:8080 while the webapp is listenning on localhost:4000
