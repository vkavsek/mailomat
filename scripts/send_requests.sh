#!/bin/bash

for ((i = 0; i < 50; i++)); do
	name="Ludi Doe $i"
	email="ludoooo.$i@example.com"

	curl -X POST \
		http://localhost:8000/api/subscribe \
		-H 'Content-Type: application/json' \
		-d '{
		"name": "'"$name"'",
		"email": "'"$email"'"
		}'
	echo "Sending Request number: $i — $name / $email"
done
