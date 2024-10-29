To ensure this works:

1. Make sure you're mounting the script correctly in docker-compose.yml:

```yml
services:
  postgres:
    # ... other config ...
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
```

2. If you still get permission errors, you might need to recreate the containers:

```bash
# Remove everything including volumes
docker-compose down -v

# Start fresh
docker-compose up -d
```

3. Verify the user was created:

```bash
# Connect to postgres container
docker exec -it postgres-db psql -U postgres

# List users
\du
```

The initialization script should run automatically when
the container first starts, creating the user with the
necessary permissions.
