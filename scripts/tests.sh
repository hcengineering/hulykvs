#!/bin/bash

clear

URL="http://localhost:8094/api2"
TOKEN=$(./token.sh claims.json)

echo ${TOKEN}

R='\033[0;31m' # Color red
G='\033[0;32m' # Color green
W='\033[0;33m' # Color ?
S='\033[0;34m' # Color Blue
F='\033[0;35m' # Color Fiolet
L='\033[0;36m' # Color LightBlue
N='\033[0m' # No Color
GRAY='\033[90m' # bright black

dbview() {
    local CN="postgresql://root@huly.local:26257/defaultdb?sslmode=disable"
    local QUERY="SELECT workspace, namespace, key, convert_from(value,'UTF8') AS value, encode(md5,'hex') AS md5 FROM hulykvs.kvs ORDER BY namespace, key;"
    local result
    if [ -z "$1" ]; then psql "$CN" -c "$QUERY"
    else
        #result=$(psql "$CN" -c "$QUERY" | grep --color=always -i "$1")
        result=$(psql "$CN" -c "$QUERY" | grep -i "$1")
        if [ -z "$result" ]; then echo -e "${R} ${1} -- not found${N}"
        else echo -e "${G}${result}${N}"
        fi
    fi
}

api() {
  local tmpfile
  tmpfile=$1
  local status
  status=$(head -n 1 "$tmpfile")
  local status_code
  status_code=$(echo "$status" | awk '{print $2}')
  local etag
  etag=$(grep -i "^ETag:" "${tmpfile}")
  local body
  body=$(awk 'found { print; next } NF == 0 { found = 1 }' "$tmpfile")
  case "$status_code" in
	2*) echo -en "${G}${status}${N}" ;;
	3*) echo -en "${F}${status}${N}" ;;
	4*) echo -en "${R}${status}${N}" ;;
	5*) echo -en "${R}${status}${N}" ;;
	*)  echo -en "${GRAY}${status}${N}" ;;
  esac
  if [ -n "$etag" ]; then echo -n -e " ${F}${etag}${N}" ; fi
  if [ -n "$body" ]; then echo -e "\n   ${GRAY}[${body}]${N}" ; else echo -e " ${L}(no body)${N}" ; fi
  rm -f "$tmpfile"
}

get() {
  echo -n -e "📥 ${L}GET ${W}$1${N} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X GET "$URL/$1" -H "Authorization: Bearer ${TOKEN}" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}

put() { # If-None-Match If-Match
  local match
  local match_prn
  if [ -n "$3" ]; then match=(-H "$3: $4") ; else match=() ; fi
  if [ -n "$3" ]; then match_prn=" ${F}$3:$4${N}" ; else match_prn="" ; fi
  echo -n -e "📥 ${L}PUT ${W}$1${N}${match_prn} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X PUT "$URL/$1" -H "Authorization: Bearer ${TOKEN}" "${match[@]}" -H "Content-Type: application/json" -d "$2" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}

delete() {
  echo -n -e "📥 ${L}DELETE ${W}$1${N} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X DELETE "$URL/$1" -H "Authorization: Bearer ${TOKEN}" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}


#      T E S T S

ZP="00000000-0000-0000-0000-000000000001/TESTS/AnyKey"

echo "================> WORKSPACEIN TOKEN (Expected ERROR 403 Forbidden)"
TOKEN=$(./token.sh claims_wrong_ws.json) # wrong token
    put ${ZP} "my value"
    get ${ZP}
    delete ${ZP}
    get "00000000-0000-0000-0000-000000000001/TESTS"

TOKEN=$(./token.sh claims.json) # restore token

echo "================> LIST"
    put "00000000-0000-0000-0000-000000000001/Huome2/MyKey1" "value1"
    put "00000000-0000-0000-0000-000000000001/Huome2/MyKey2" "value2"
    get "00000000-0000-0000-0000-000000000001/Huome2"
    delete "00000000-0000-0000-0000-000000000001/Huome2/MyKey1"
    delete "00000000-0000-0000-0000-000000000001/Huome2/MyKey2"

echo "================> WRONG UUID"
    get "WrongUUID/TESTS/AnyKey"

echo "================> INSERT If-None-Match"

    echo "-- Expected Error: 400 Bad Request (If-None-Match may be only *)"
     put ${ZP} "enother text" "If-None-Match" "552e21cd4cd9918678e3c1a0df491bc3"

    delete ${ZP}

    echo "-- Expected OK: 201 Created (key was not exist)"
     put ${ZP} "enother text" "If-None-Match" "*"

    put ${ZP} "some text"
    echo "-- Expected Error: 412 Precondition Failed (key was exist)"
     put ${ZP} "enother text" "If-None-Match" "*"

echo "================> UPDATE PUT If-Match"

    get ${ZP}

    echo "-- Expected OK: 204 No Content (right hash)"
     put ${ZP} "some text" "If-Match" "552e21cd4cd9918678e3c1a0df491bc3"
    get ${ZP}

    echo "-- Expected OK: 204 No Content (hash still right)"
     put ${ZP} "enother version" "If-Match" "552e21cd4cd9918678e3c1a0df491bc3"
    get ${ZP}

    echo "-- Expected OK: 204 No Content (any hash)"
     put ${ZP} "enother version2" "If-Match" "*"
    get ${ZP}

    echo "-- Expected Error: 412 Precondition Failed (wrong hash)"
     put ${ZP} "enother version3" "If-Match" "552e21cd4cd9918678e3c1a0df491bc3"

    delete ${ZP}

    echo "-- Expected Error: 412 Precondition Failed (any hash not found)"
     put ${ZP} "enother version2" "If-Match" "*"

echo "================> UPSERT (Expected OK)"
    put ${ZP} "my value"
    get ${ZP}
    put ${ZP} "my new value"
    get ${ZP}

exit
