#!/bin/bash

clear

NS="TESTS"
KEY="keylleo2"
VALUE="{\"name\": \"John Fox\", \"penis\": \"$(( RANDOM % 20 + 5 ))\"}"
TOKEN=$(./token.sh lleo)

# read all (GET)
echo
echo -n "📥 GET /api/${NS} = "
curl -s -X GET "http://localhost:8094/api/${NS}" -H "Authorization: Bearer $TOKEN"

# write (POST)
curl -s -o /dev/null -w "✅ Stored key '%s' in namespace '%s' → HTTP %s\n" \
  -X POST "http://localhost:8094/api/$NS/$KEY" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$VALUE" \
  --write-out "📥 POST(%{http_code}) /api/$NS/$KEY = $VALUE\n"

# read (GET)
echo
echo -n "📥 GET /api/$NS/$KEY = "
curl -s -X GET "http://localhost:8094/api/$NS/$KEY" -H "Authorization: Bearer $TOKEN"
# | jq .

# read all (GET)
echo
echo -n "📥 GET /api/${NS} = "
curl -s -X GET "http://localhost:8094/api/${NS}" -H "Authorization: Bearer $TOKEN"
