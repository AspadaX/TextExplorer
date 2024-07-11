
# TextExplorer

<div align="center">
   <img src="assets/explorer-dark.jpg" alt="logo" width="250">
</div>

TextExplorer is a powerful tool designed to make reading and searching through massive texts easier. It leverages vector search technology to efficiently find relevant information within large documents. This project uses the Qdrant vector database and ONNX for embedding models to provide fast and accurate search results.

The backend is written in Rust for high performance. 

The open source version of this project is only for **RESEARCH** purposes. 

If you are interested in this project and would like to join, please reach me out. Commercial interests are also welcomed: 

Discord: `https://discord.gg/7u9U2Es5`

WeChat: `baoxinyu2007`

## Features

- **Semantic Search:** Utilizes advanced vector search to quickly locate relevant information in large texts.
- **User Interface:** Provides a rudimentary UI for interacting with the text data, adding new documents, and managing collections.
- **Document Management:** Allows for the addition and deletion of document collections.

## Installation

### Prerequisites

- Docker (for running Qdrant and easier access to the project)

### Setup
1. **Prepare the embedding model:**
   
   You will need to download a copy of `BAAI/bge-m3` on Huggingface, and place it under: `./onnx`

#### Using Docker Compose
1. Ensure Docker Compose is installed.

2. Run Docker Compose:

```sh
docker compose up
```
This will start both the TextExplorer application and the Qdrant service. And it should be accessible via `localhost:3000`

Note: If you would like to run the docker container in the background, please add `-d` at the end of the above command like so: `docker compose up -d`

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
  
## Roadmap

- Implement in Rust for blazingly fast performance. (Done)
- Implement a better looking interface. (Done)
- Add more practical features, for example, parsing files, videos/audios and websites. (In-Progress)
- Launch a web service for people without a tech background to hand on easily.  
  
## Credits

- [Qdrant](https://github.com/qdrant/qdrant): A distributed, high-performance vector search engine.
- [BAAI/bge-m3](https://huggingface.co/BAAI/bge-m3): A pre-trained Chinese language model for text embedding.
- [benbrandt/text-splitter](https://github.com/benbrandt/text-splitter): Split text into semantic chunks, up to a desired chunk size. Supports calculating length by characters and tokens, and is callable from Rust and Python.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed for research purposes only. For commercial use, please contact the project owner.