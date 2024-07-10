import React, { useState, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import styled from "styled-components";


const Container = styled.div`
  display: flex;
  height: 100vh;
  background-color: var(--background-color);
  color: var(--text-color);
`;

const SidebarStyle = styled.div`
  flex: 1;
  background-color: var(--sidebar-bg-color);
  padding: 20px;
  overflow: auto;
  flex: 0 0 250px;
`;

const Content = styled.div`
  flex: 3;
  padding: 20px;
  overflow: auto;
`;

const SearchContainer = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: var(--sidebar-bg-color);
  padding: 20px;
  overflow: auto;
`;

const SearchBar = styled.div`
  margin-bottom: 20px;
`;

const Results = styled.div`
  flex: 1;
  overflow: auto;
`;

const ListItem = styled.li`
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
    span {
        flex: 1;
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        margin-right: 10px;
    }
    button {
        padding: 5px 10px;
        background-color: #ff4d4f;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
    }
`;

function App() {
    const [selectedDocumentId, setSelectedDocumentId] = useState(null);
    const [documents, setDocuments] = useState([]);
    const [text, setText] = useState('');
    const [title, setTitle] = useState('');

    useEffect(() => {
        if (selectedDocumentId) {
            fetch(`http://localhost:10000/document/read/${selectedDocumentId}`)  // Ensure this URL is correct
                .then(response => {
                    if (!response.ok) {
                        throw new Error('Network response was not ok');
                    }
                    return response.json();
                })
                .then(data => {
                    if (data.status && data.data) {
                        setTitle(data.data.title);
                        setText(data.data.full_text); // Ensure these properties exist in your data
                    } else {
                        throw new Error('Failed to load document');
                    }
                })
                .catch(err => {
                    console.error("Fetching document error:", err);
                    alert(err.message);
                });
        } else {
            setTitle('');
            setText(''); // Reset text and title when no document is selected
        }
    }, [selectedDocumentId]);  // Dependency array includes selectedDocumentId

    // Fetch documents whenever there's a change indicating a need for refresh
    useEffect(() => {
        fetchDocuments();
    }, [selectedDocumentId]);

    const fetchDocuments = () => {
        fetch('http://localhost:10000/document/read/get_all_documents')
            .then(response => response.json())
            .then(data => {
                if (data.status) {
                    setDocuments(data.data);
                } else {
                    console.error('Failed to fetch documents:', data.message);
                }
            })
            .catch(err => console.error("Fetching documents error:", err));
    };

    // Function to handle document deletion
    const deleteDocument = (documentId) => {
        fetch(`http://localhost:10000/document/delete`, {
            method: 'DELETE',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ document_id: documentId })
        })
        .then(response => response.json())
        .then(data => {
            if (data.status) {
                alert('Document was successfully deleted.');
                if (selectedDocumentId === documentId) {
                    setSelectedDocumentId(null);
                    setText('');
                    setTitle('');
                }
                fetchDocuments();  // Refresh documents after deletion
            } else {
                console.error('Error deleting document:', data.message);
            }
        })
        .catch(error => console.error('Delete error:', error));
    };


    return (
        <Container>
            <Sidebar 
                documents={documents} 
                setSelectedDocumentId={setSelectedDocumentId} 
                deleteDocument={deleteDocument}
            />
            <Content>
                <Textbox
                    documentId={selectedDocumentId}
                    setDocumentId={setSelectedDocumentId}
                    text={text}
                    setText={setText}
                    title={title}
                    setTitle={setTitle}
                    fetchDocuments={fetchDocuments}
                />
                <button onClick={() => setSelectedDocumentId(null)}>New Document</button>
                <MarkdownDisplay text={text}></MarkdownDisplay>
            </Content>
            <SearchContainer>
                {selectedDocumentId && <Search documentId={selectedDocumentId} />}
            </SearchContainer>
        </Container>
    );
}


function MarkdownDisplay({text}) {
    return (
        <div className='markdown-container'>
            <ReactMarkdown>{text}</ReactMarkdown>
        </div>
    );
}

function Textbox({ documentId, setDocumentId, text, setText, title, setTitle, fetchDocuments }) {
    const [isLoading, setIsLoading] = useState(false);

    const handleSave = () => {
        let apiURL, body;

        if (documentId) {
            // Update existing document
            apiURL = `http://localhost:10000/document/update`;
            body = JSON.stringify({
                updated_parameters: {
                    title: title,
                    full_text: text,
                    maximum_text_split_size: 256
                },
                document_id: documentId
            });
        } else {
            // Create new document
            apiURL = `http://localhost:10000/document/create`;
            body = JSON.stringify({
                title: title,
                full_text: text,
                maximum_text_split_size: 256
            });
        }
        setIsLoading(true);
        fetch(apiURL, {
            method: "POST",
            headers: { 'Content-Type': 'application/json' },
            body: body
        })
            .then(response => {
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                return response.json();
            })
            .then(data => {
                if (data.status) {
                    alert('Document saved successfully');
                    if (!documentId) {
                        setDocumentId(data.document_id);  // Update the document ID for new documents
                    }
                } else {
                    throw new Error(data.message);
                }
                setIsLoading(false);
            })
            .catch(error => {
                console.error('Error:', error);
                alert(error.message);
                setIsLoading(false);
            });
    };

    return (
        <div>
            <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Enter title" />
            <textarea value={text} onChange={(e) => setText(e.target.value)} placeholder="Enter text here..." />
            <button onClick={handleSave} disabled={isLoading}>
                {documentId ? 'Update Document' : 'Create Document'}
            </button>
        </div>
    );
}


function Search({ documentId }) {
    const [inputText, setInputText] = useState('');
    const [searchResults, setSearchResults] = useState([]);
    const [loading, setLoading] = useState(false);

    const handleSearch = async () => {
        setLoading(true);
        const requestOptions = {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ document_id: documentId, input_text: inputText, top_n: 10 })
        };
        try {
            const response = await fetch("http://localhost:10000/document/read/vector_search", requestOptions);
            const data = await response.json();
            if (data.status) {
                setSearchResults(data.data);
            } else {
                throw new Error(data.message);
            }
        } catch (error) {
            console.error('Search failed:', error);
        } finally {
            setLoading(false);
        }
    };

    return (
        <SearchContainer>
            <SearchBar>
                <input
                    type="text"
                    value={inputText}
                    onChange={(e) => setInputText(e.target.value)}
                    placeholder="Search query"
                />
                <button onClick={handleSearch}>Comprehend</button>
            </SearchBar>
            <Results>
                {loading && <div>Loading results...</div>}
                <ul>
                    {searchResults.map((result, index) => (
                        <li key={index}>{result.relevance} - {result.content}</li>
                    ))}
                </ul>
            </Results>
        </SearchContainer>
    );
}

function Sidebar({ setSelectedDocumentId, documents, deleteDocument }) {
    // Remove internal state management for documents, loading, and error,
    // assuming these are handled by the parent component.

    return (
        <SidebarStyle>
            <h2>Documents</h2>
            {documents.length > 0 ? (
                <ul>
                    {documents.map(document => (
                        <ListItem key={document.id} onClick={() => setSelectedDocumentId(document.id)}>
                            <body>
                                {document.title}
                            </body>
                            <button
                                style={
                                    { 
                                        width: '25px', 
                                        height: '25px', 
                                        textAlign: 'center'
                                    }
                                }
                                onClick={() => deleteDocument(document.id)}
                            >X</button>
                        </ListItem>
                    ))}
                </ul>
            ) : (
                <div>No documents available</div>  // Proper feedback
            )}
        </SidebarStyle>
    );
}
export default App;