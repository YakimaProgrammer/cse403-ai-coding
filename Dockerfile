# Build Stage for Frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
ARG VITE_API_URL
ENV VITE_API_URL=${VITE_API_URL:-http://127.0.0.1:8000}
RUN npm run build

# Final Stage for Backend
FROM python:3.11-slim
WORKDIR /app

# Install system dependencies for OR-Tools/SCIP
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Copy backend and install dependencies
COPY backend/requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY backend/ ./

# Accept API_BASE_PATH as a build argument or runtime environment variable
ARG API_BASE_PATH
ENV API_BASE_PATH=${API_BASE_PATH:-}

# Copy built frontend assets from builder stage
# These will be in /app/dist inside the container
COPY --from=frontend-builder /app/frontend/dist ./static_build

EXPOSE 8000

# Run the FastAPI app
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
