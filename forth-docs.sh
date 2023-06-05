#!/usr/bin/env bash
dir="forth-standard.org"
base_site="https://$dir"
topics=$(curl --silent "$base_site/standard/words" | htmlq '.boxList' --attribute href a)
for topic in $topics
do
  echo "TOPIC: $topic"
  echo -n "" > "$dir/${topic/'/standard/'}.data"
  words=$(curl --silent "$base_site$topic" | htmlq '#body' --attribute href a)
  for word in $words
  do
    word="${word//core/}"
    if [[ "$word" != *"#"* ]]; then
      id=$(curl --silent "$base_site$topic$word" | htmlq -p  --text '.forth-200x-wrapper > h1 > .name')
      help=$(curl --silent "$base_site$topic$word"| htmlq --text '.forth-200x-wrapper div' | rg -m 1 '\(.*\)' | sed -E 's/^ //g')
      description=$(curl --silent "$base_site$topic$word" | htmlq -p  --text '.forth-200x-wrapper > div > p:first-of-type' | tr -d '\n' | sed -E 's/ +/ /g' | sed -E 's/^ //g')

      printf "%s\n%s\n%s\n%s\n\n" "$word" "$id" "$help" "$description" >> "$dir/${topic/'/standard/'}.data"
    fi
  done
done

