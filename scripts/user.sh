# update user age
curl -X PATCH http://localhost:8080/update_user \
-H "Content-Type: application/json" \
-d '{"age": 30}'

# update user skills
curl -X PATCH http://localhost:8080/update_user \
-H "Content-Type: application/json" \
-d '{"skills": ["JavaScript", "Python", "React"]}'
