# Dockerfile for Python Backend
FROM python:3.11.4-slim

# Set the working directory
WORKDIR /app

# Copy the Python requirements file
COPY requirements.txt .

# Set a specific PyPI mirror for pip
ARG PIP_INDEX_URL=https://pypi.tuna.tsinghua.edu.cn/simple/

# Install Python dependencies
RUN pip install --trusted-host pypi.tuna.tsinghua.edu.cn \
    --index-url ${PIP_INDEX_URL} --no-cache-dir -r requirements.txt

# Copy the backend code
COPY . .

# Expose the port the backend runs on
EXPOSE 3000

# Command to run the backend
CMD ["python", "GUI.py"]