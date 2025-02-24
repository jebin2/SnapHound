import { useEffect, useState, useCallback, memo, useRef } from "react";
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from "@tauri-apps/api/event";
import uploadIcon from "./assets/upload_icon.png";
import settingIcon from "./assets/setting_icon.png";
import "./App.css";

// Constants for search types to avoid magic strings
const SEARCH_TYPES = {
  IMAGE: "image",
  VIDEO: "video"
};

// Memoized Media Item Component with custom comparison
const MediaItem = memo(({ item }) => {
  const imgRef = useRef(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => setIsVisible(entry.isIntersecting),
      { threshold: 0.1 }
    );
    
    const currentRef = imgRef.current;
    currentRef && observer.observe(currentRef);

    return () => currentRef && observer.unobserve(currentRef);
  }, []);

  const imgSrc = isVisible ? convertFileSrc(item.path) : null;

  if (item.id === "empty") {
    return <div className="media-item" ref={imgRef}>No media found</div>;
  }

  return (
    <div className="media-item" ref={imgRef}>
      {item.type === "video" && <div className="video-indicator">▶</div>}
      {imgSrc && (<img
        id={item.id}
        src={imgSrc}
        alt={item.name}
        loading="lazy"
        style={{ width: "200px", height: "200px" }}
        onError={(e) => (e.target.style.display = "none")}
      />)}
    </div>
  );
}, (prev, next) => (
  prev.item.id === next.item.id &&
  prev.item.path === next.item.path &&
  prev.item.type === next.item.type &&
  prev.item.name === next.item.name
));

// Settings Modal Component
const SettingsModal = memo(({ 
  searchPaths, 
  onClose, 
  onAddPath, 
  onRemovePath, 
  onSave 
}) => (
  <div className="modal">
    <div className="modal-content">
      <h2>Manage Search Paths</h2>
      <div className="path-list">
        {searchPaths.map((path, index) => (
          <div key={`${path}-${index}`} className="path-item">
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
));

// Video Modal Component
const VideoModal = memo(({ onClose }) => (
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
));

// Header Component
const Header = memo(({ 
  searchType, 
  onSearchTypeChange, 
  openSettings, 
  openVideoModal 
}) => (
  <header className="search-header">
    <div className="search-box">
      <input 
        type="text" 
        className="search-input" 
        placeholder="Search with text or image..." 
      />
      <img
        className="upload_icon"
        src={uploadIcon}
        alt="Upload"
        style={{ width: "40px" }}
      />
    </div>
    <div className="search-type">
      <button
        className={`search-type-btn ${searchType === SEARCH_TYPES.IMAGE ? "active" : ""}`}
        onClick={() => onSearchTypeChange(SEARCH_TYPES.IMAGE)}
      >
        Images
      </button>
      <button
        className={`search-type-btn ${searchType === SEARCH_TYPES.VIDEO ? "active" : ""}`}
        onClick={() => {
          onSearchTypeChange(SEARCH_TYPES.VIDEO);
          openVideoModal();
        }}
      >
        Videos
      </button>
      <img
        className="setting_icon"
        src={settingIcon}
        alt="Settings"
        style={{ width: "40px" }}
        onClick={openSettings}
      />
    </div>
  </header>
));

// Main App Component
function App() {
  const [searchType, setSearchType] = useState(SEARCH_TYPES.IMAGE);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [videoModalOpen, setVideoModalOpen] = useState(false);
  const [mediaItems, setMediaItems] = useState([]);
  const [searchPaths, setSearchPaths] = useState([]);

  // Tauri API Wrapper with stable reference
  const invokeAPI = useCallback(async (method, args) => {
    try {
      return await invoke(method, args);
    } catch (error) {
      alert(error.message || "An error occurred");
      throw error;
    }
  }, []);

  // Environment initialization
  useEffect(() => {
    invokeAPI("initialize_environment");
  }, [invokeAPI]);

  // Event listeners setup
  useEffect(() => {
    const unlisten = [];
    
    const setupListeners = async () => {
      try {
        unlisten.push(await listen("file_path", ({ payload }) => {
          const newItems = JSON.parse(payload);
          setMediaItems(prev => [
            ...new Map([...prev, ...newItems].map(item => [item.id, item])).values()
          ]);
        }));

        unlisten.push(await listen("file_path_end", () => {
          setMediaItems(prev => prev.length ? prev : [{ id: "empty" }]);
        }));

        unlisten.push(await listen("can_fetch_list", () => {
          invokeAPI("list_files");
        }));
      } catch (error) {
        console.error("Error setting up listeners:", error);
      }
    };

    setupListeners();
    return () => unlisten.forEach(fn => fn?.());
  }, [invokeAPI]);

  // Settings management
  const fetchConfig = useCallback(async () => {
    try {
      const config = await invokeAPI("fetch_config");
      setSearchPaths(config?.priority_path || []);
      setSettingsOpen(true);
    } catch (error) {
      console.error("Failed to fetch config:", error);
    }
  }, [invokeAPI]);

  const saveConfig = useCallback(async () => {
    try {
      await invokeAPI("save_config", { priorityPath: searchPaths });
      setSettingsOpen(false);
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  }, [invokeAPI, searchPaths]);

  const handleAddPath = useCallback(async () => {
    try {
      const folderPath = await invokeAPI("select_folder");
      folderPath && setSearchPaths(prev => [...prev, folderPath]);
    } catch (error) {
      console.error("Failed to select folder:", error);
    }
  }, [invokeAPI]);

  // Event handlers with stable references
  const handleSearchTypeChange = useCallback((type) => {
    setSearchType(type);
    if (type === SEARCH_TYPES.VIDEO) setVideoModalOpen(true);
  }, []);

  const handleOpenSettings = useCallback(() => fetchConfig(), [fetchConfig]);
  const handleCloseSettings = useCallback(() => setSettingsOpen(false), []);
  const handleRemovePath = useCallback((index) => 
    setSearchPaths(prev => prev.filter((_, i) => i !== index)),
  []);
  const handleCloseVideoModal = useCallback(() => setVideoModalOpen(false), []);

  return (
    <div className="container">
      <Header
        searchType={searchType}
        onSearchTypeChange={handleSearchTypeChange}
        openSettings={handleOpenSettings}
        openVideoModal={() => setVideoModalOpen(true)}
      />

      <div className="progress-bar">
        <div className="progress-fill" />
      </div>

      <main className="media-container">
        <div className="media-grid">
          {mediaItems.map(item => (
            <MediaItem key={item.id} item={item} />
          ))}
        </div>
      </main>

      {videoModalOpen && <VideoModal onClose={handleCloseVideoModal} />}
      
      {settingsOpen && (
        <SettingsModal
          searchPaths={searchPaths}
          onClose={handleCloseSettings}
          onAddPath={handleAddPath}
          onRemovePath={handleRemovePath}
          onSave={saveConfig}
        />
      )}
    </div>
  );
}

export default App;