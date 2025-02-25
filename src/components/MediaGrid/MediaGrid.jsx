import React from 'react';
import MediaItem from './MediaItem';
import '../../styles/MediaGrid.css';

/**
 * MediaGrid component to display a grid of media items
 */
const MediaGrid = ({ mediaItems }) => {
  return (
    <main className="media-container">
      <div className="media-grid">
        {mediaItems.map(item => (
          <MediaItem key={item.id} item={item} />
        ))}
      </div>
    </main>
  );
};

export default MediaGrid;