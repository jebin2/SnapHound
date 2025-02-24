import { useEffect, useState, useCallback, memo, useRef } from "react";
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from "@tauri-apps/api/event";
import uploadIcon from "./assets/upload_icon.png";
import settingIcon from "./assets/setting_icon.png";
import "./App.css";

// Memoized Media Item Component
const MediaItem = memo(({ item }) => {
  const imgRef = useRef(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { threshold: 0.1 }
    );

    const currentRef = imgRef.current;
    if (currentRef) observer.observe(currentRef);

    return () => {
      if (currentRef) observer.unobserve(currentRef);
    };
  }, []);

  // Only convert the file path when the component is visible
  const imgSrc = isVisible ? convertFileSrc(item.path) : '';

  if (item.id === "empty") {
    return <div className="media-item" ref={imgRef}>No media found</div>;
  }

  return (
    <div className="media-item" ref={imgRef}>
      {item.type === "video" && <div className="video-indicator">▶</div>}
      <img
        id={item.id}
        src={imgSrc}
        alt={item.name}
        loading="lazy"
        style={{ width: "200px", height: "200px" }}
        onError={(e) => (e.target.style.display = "none")}
      />
    </div>
  );
});

// Settings Modal Component
const SettingsModal = memo(({ 
  searchPaths, 
  onClose, 
  onAddPath, 
  onRemovePath, 
  onSave 
}) => {
  return (
    <div className="modal">
      <div className="modal-content">
        <h2>Manage Search Paths</h2>
        <div className="path-list">
          {searchPaths.map((path, index) => (
            <div key={path + index} className="path-item">
              <input className="path-input" value={path} readOnly />
              <button 
                className="remove-path" 
                onClick={() => onRemovePath(index)}
              >
                ✕
              </button>
            </div>
          ))}
        </div>

        <button className="add-path-btn" onClick={onAddPath}>
          ＋ Add New Path
        </button>

        <div className="modal-actions">
          <button className="search-type-btn" onClick={onClose}>
            Cancel
          </button>
          <button className="search-type-btn" onClick={onSave}>
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
});

// Video Modal Component
const VideoModal = memo(({ onClose }) => {
  return (
    <div className="modal">
      <div className="modal-content">
        <h2>Video Search</h2>
        <p>This may take some time. Select videos to analyze:</p>
        <button className="start-analysis">Start Analysis</button>
        <button className="close-modal" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
});

// Main App Component
function App() {
  // State management
  const [searchType, setSearchType] = useState("image");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [videoModalOpen, setVideoModalOpen] = useState(false);
  const [mediaItems, setMediaItems] = useState([]);
  const [searchPaths, setSearchPaths] = useState([]);

  /** Tauri API Wrapper */
  const invokeAPI = useCallback(async (method, args) => {
    try {
      return await invoke(method, args);
    } catch (error) {
      console.error(`Error invoking ${method}:`, error);
      alert(error.message || "An error occurred");
      throw error;
    }
  }, []);

  /** Initialize environment on load */
  useEffect(() => {
    invokeAPI("initialize_environment");
  }, [invokeAPI]);

  /** Set up event listeners */
  useEffect(() => {
    const unlisten = [];

    const setupListeners = async () => {
      // Listen for file_path events
      const unlistenFilePath = await listen("file_path", (event) => {
        try {
          const newItems = JSON.parse(event.payload);
          setMediaItems((prev) => {
            // Use Map to ensure uniqueness by ID
            const uniqueItems = [...new Map(
              [...prev, ...newItems].map(item => [item.id, item])
            ).values()];
            return uniqueItems;
          });
        } catch (e) {
          console.error("Error parsing file_path payload", e);
        }
      });
      unlisten.push(unlistenFilePath);

      // Listen for file_path_end events
      const unlistenFilePathEnd = await listen("file_path_end", () => {
        setMediaItems((prev) => (
          prev.length === 0 ? [{ name: "No media found", id: "empty" }] : prev
        ));
      });
      unlisten.push(unlistenFilePathEnd);

      // Listen for can_fetch_list events
      const unlistenFetchList = await listen("can_fetch_list", () => {
        invokeAPI("list_files");
      });
      unlisten.push(unlistenFetchList);
    };

    setupListeners();

    // Cleanup function
    return () => {
      unlisten.forEach(fn => {
        if (typeof fn === 'function') fn().catch(err => console.error("Error unlistening", err));
      });
    };
  }, [invokeAPI]);

  /* Settings handlers */
  const openSettings = useCallback(async () => {
    try {
      const config = await invokeAPI("fetch_config");
      setSearchPaths(config?.priority_path || []);
      setSettingsOpen(true);
    } catch (error) {
      console.error("Failed to fetch config:", error);
    }
  }, [invokeAPI]);

  const closeSettings = useCallback(() => {
    setSettingsOpen(false);
  }, []);

  const addSearchPath = useCallback(async () => {
    try {
      const folderPath = await invokeAPI("select_folder");
      if (folderPath) {
        setSearchPaths(prevPaths => [...prevPaths, folderPath]);
      }
    } catch (error) {
      console.error("Failed to select folder:", error);
    }
  }, [invokeAPI]);

  const removeSearchPath = useCallback((index) => {
    setSearchPaths(prevPaths => prevPaths.filter((_, i) => i !== index));
  }, []);

  const saveSearchPaths = useCallback(async () => {
    try {
      await invokeAPI("save_config", { priorityPath: searchPaths });
      closeSettings();
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  }, [invokeAPI, searchPaths, closeSettings]);

  /* UI Event Handlers */
  const handleSearchTypeClick = useCallback((type) => {
    setSearchType(type);
  }, []);

  const openVideoModal = useCallback(() => {
    setVideoModalOpen(true);
  }, []);

  const closeVideoModal = useCallback(() => {
    setVideoModalOpen(false);
  }, []);

  return (
    <div className="container">
      <header className="search-header">
        <div className="search-box">
          <input 
            type="text" 
            className="search-input" 
            placeholder="Search with text or image..." 
          />
          <img
            id="upload_icon"
            className="upload_icon"
            src={uploadIcon}
            alt="Upload Icon"
            style={{ width: "40px" }}
          />
        </div>
        <div className="search-type">
          <button
            className={`search-type-btn ${searchType === "image" ? "active" : ""}`}
            onClick={() => handleSearchTypeClick("image")}
          >
            Images
          </button>
          <button
            className={`search-type-btn ${searchType === "video" ? "active" : ""}`}
            onClick={openVideoModal}
          >
            Videos
          </button>
          <img
            id="setting_icon"
            className="setting_icon"
            src={settingIcon}
            alt="Settings Icon"
            style={{ width: "40px" }}
            onClick={openSettings}
          />
        </div>
      </header>

      <div className="progress-bar">
        <div className="progress-fill"></div>
      </div>

      <main className="media-container">
        <div className="media-grid">
          {mediaItems.map((item) => (
            <MediaItem key={item.id} item={item} />
          ))}
        </div>
      </main>

      {/* Modals */}
      {videoModalOpen && <VideoModal onClose={closeVideoModal} />}
      
      {settingsOpen && (
        <SettingsModal
          searchPaths={searchPaths}
          onClose={closeSettings}
          onAddPath={addSearchPath}
          onRemovePath={removeSearchPath}
          onSave={saveSearchPaths}
        />
      )}
    </div>
  );
}

export default App;