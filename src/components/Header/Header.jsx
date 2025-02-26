import React from 'react';
import { SEARCH_TYPES } from '../../constants';
import uploadIcon from '../../assets/upload_icon.png';
import settingIcon from '../../assets/setting_icon.png';
import '../../styles/Header.css';

/**
 * Header component with search box and type selection
 */
const Header = ({ 
  onSearchInputChange,
  searchType, 
  searchValue, 
  onSearchTypeChange, 
  openSettings, 
  openVideoModal 
}) => (
  <header className="search-header">
    <div className="search-box">
      <input 
        type="text" 
        value={searchValue}
        className="search-input" 
        placeholder="Search with text or image..."
        onChange={onSearchInputChange} 
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
);

export default React.memo(Header);