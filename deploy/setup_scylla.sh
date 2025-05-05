#!/bin/bash
docker run -d \
  --name scylla \
  -p 9042:9042 \
  -v scylla-data:/var/lib/scylla \
  scylladb/scylla
