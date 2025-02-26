import { useState, useCallback } from "react";
import MediaItem from './MediaItem';
import '../../styles/MediaGrid.css';
import { ViewImageModal } from "../Modals";
import { convertFileSrc } from '@tauri-apps/api/core';

/**
 * MediaGrid component to display a grid of media items
 */
const MediaGrid = ({ mediaItems }) => {
  const [viewImageModalOpen, setViewImageModalOpen] = useState(false); // Corrected: Initial state to false
  const [viewImagePath, setViewImagePath] = useState("");
  const handleCloseViewImageModal = useCallback(() => setViewImageModalOpen(false), []);

  // Corrected: onImageClick now receives the item as argument
  const onImageClick = useCallback((item) => {
    console.log(convertFileSrc(item.file_path));
    setViewImagePath(convertFileSrc(item.file_path)); // Set the path of the clicked item
		setViewImageModalOpen(true);
	}, []);

  return (
    <main className="media-container">
      <div className="media-grid">
        {mediaItems.map(item => (
          <MediaItem key={item.id} item={item} onImageClick={onImageClick} />
        ))}
      </div>
      {viewImageModalOpen && <ViewImageModal onClose={handleCloseViewImageModal} viewImagePath={viewImagePath} />}
    </main>
  );
};

export default MediaGrid;