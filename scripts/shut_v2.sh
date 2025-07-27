#!/bin/bash

clear

WS="Huone"
NS="TESTS"
KEY="AnyKey"
VALUE="{\"name\": \"Pavel\", \"penis\": \"$(( RANDOM % 20 + 5 ))\"}"
TOKEN=$(./token.sh lleo)

# read all (GET)
echo
echo -n "📥 GET /api2/${WS}/${NS} = "
curl -s -X GET "http://localhost:8094/api2/${WS}/${NS}" -H "Authorization: Bearer $TOKEN"

# write (POST)
curl -s -o /dev/null -w "✅ Stored key '%s' in namespace '%s' → HTTP %s\n" \
  -X POST "http://localhost:8094/api2/${WS}/$NS/$KEY" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$VALUE" \
  --write-out "📥 POST(%{http_code}) /api2/${WS}/$NS/$KEY = $VALUE\n"

# read (GET)
echo
echo -n "📥 GET /api2/${WS}/$NS/$KEY = "
curl -s -X GET "http://localhost:8094/api2/${WS}/$NS/$KEY" -H "Authorization: Bearer $TOKEN"
# | jq .

# read all (GET)
echo
echo -n "📥 GET /api2/${WS}/${NS} = "
curl -s -X GET "http://localhost:8094/api2/${WS}/${NS}" -H "Authorization: Bearer $TOKEN"

# read all ?prefix=keyl (GET)
echo
echo -n "📥 GET /api2/${WS}/${NS}?prefix=keyl = "
curl -s -X GET "http://localhost:8094/api2/${WS}/${NS}?prefix=keyl" -H "Authorization: Bearer $TOKEN"
