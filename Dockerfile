# Start from a minimal base image for running the compiled binaries
FROM ubuntu:22.04

# Install NGINX to serve the frontend
RUN apt-get update && \
    apt-get install -y nginx libsqlite3-dev libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/* && \
    rm /etc/nginx/sites-enabled/default

# Set up directories
WORKDIR /app

# Copy the pre-built Rust backend binary and other necessary files
COPY ./rust_backend /app
COPY ./.env /app
COPY ./sqlite_database.db /app
COPY ./configurations.json /app

# Copy the pre-built frontend files
COPY ./build /var/www/html

# Custom NGINX configuration to serve the frontend and proxy to the backend
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose only the port for the frontend
EXPOSE 3000

# CMD to start both NGINX and your backend
CMD ["sh", "-c", "service nginx start && ./rust_backend"]