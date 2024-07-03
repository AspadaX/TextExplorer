
# TextExplorer

TextExplorer is a powerful tool designed to make reading and searching through massive texts easier. It leverages vector search technology to efficiently find relevant information within large documents. This project uses the Qdrant vector database and ONNX for embedding models to provide fast and accurate search results.

The open source version of this project is only for **RESEARCH** purposes. 

If you are interested in this project and would like to be part of this project, please reach me out. Commercial interests are also welcomed: 

Discord: `https://discord.gg/7u9U2Es5`

WeChat: `baoxinyu2007`

## Features

- **Vector Search:** Utilizes advanced vector search to quickly locate relevant information in large texts.
- **User Interface:** Provides a rudimentary UI for interacting with the text data, adding new documents, and managing collections.
- **Document Management:** Allows for the addition and deletion of document collections.

## Installation

### Prerequisites

- Python 3.11.4+ (The project is developed on 3.11.4)
- Docker (for running Qdrant and easier access to the project)
- ONNX Runtime

### Setup
1. **Prepare the embedding model:**
   
   You will need to download a copy of `BAAI/bge-m3` on Huggingface, and place it under: `../vectornotes-caches/embeddings`

2. **Clone the repository:**

   ```sh
   git clone https://github.com/your-username/TextExplorer.git
   cd TextExplorer
   ```

3. **Install the required Python packages:**

   ```sh
   pip install -r requirements.txt
   ```

4. **Run Qdrant:**

   Make sure to run Qdrant using Docker:

   ```sh
   docker run -p 6333:6333 -p 6334:6334 -v /path/to/qdrant_storage:/qdrant/storage:z qdrant/qdrant
   ```

5. **Configure the application:**

   Ensure your configuration files are properly set up. Refer to `utilities.Lifespan` for configuration management.

6. **Run the application:**

   ```sh
   python GUI.py
   ```
   Notice that the application should be run at 10000 port by default. 

### Using Docker
You can also set up and run the project using Docker and Docker Compose.

#### Build and Run with Docker
1. Build the Docker image:

```sh
docker build -t text-explorer .
```
2. Run the Docker container:

```sh
docker run -p 10000:10000 text-explorer
```
#### Using Docker Compose
1. Ensure Docker Compose is installed.

2. Run Docker Compose:

```sh
docker-compose up
```
This will start both the TextExplorer application and the Qdrant service. And it should be accessible via `localhost:10000`

## Usage

### Main Page

The main interface allows you to:

- Add new documents.
- View existing documents.
- Search through documents using vector search.

### Adding Documents

To add a document:

1. Enter the document name.
2. Input your content in the provided textarea.
3. Click the button to add the document to the collection.

### Searching Documents

Use the search functionality to input your query and get relevant results based on the vector search mechanism. 

You can use natural languages to describe your thoughts or query, for example:

- "What is the meaning of life?"
- "What is the best way to learn Python?"
- "How can I improve my writing skills?"
  
This sets the difference between the traditional keyword search and semantic search. The top ranked results should be the most relevant to your questions. 

### Managing Collections

You can delete collections as needed, which will remove all documents within that collection.

## Project Structure

- `GUI.py`: Initializes the application, loads configurations, and sets up the main page.
- `MainPage.py`: Handles the main UI interactions, including adding and searching documents.
- `UIControllers.py`: Manages UI control logic and updates.
  
## Roadmap

- Implement in Rust for blazingly fast performance. 🔥
- Implement a better looking interface. 
- Add more practical features, for example, parsing files and websites. 
- Launch a web service for people without a tech background to hand on easily.  
  
## Special Thanks

- [Qdrant](https://github.com/qdrant/qdrant): A distributed, high-performance vector search engine.
- [BAAI/bge-m3](https://huggingface.co/BAAI/bge-m3): A pre-trained Chinese language model for text embedding.
- [benbrandt/text-splitter](https://github.com/benbrandt/text-splitter): Split text into semantic chunks, up to a desired chunk size. Supports calculating length by characters and tokens, and is callable from Rust and Python.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed for research purposes only. For commercial use, please contact the project owner.
