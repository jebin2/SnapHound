import React, { useEffect, useRef, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { areMediaItemsEqual } from '../../utils/mediaUtils';
import '../../styles/MediaGrid.css';

/**
 * MediaItem component with lazy loading
 */
const MediaItem = ({ item, onImageClick }) => {
  const imgRef = useRef(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    // Set up intersection observer for lazy loading
    const observer = new IntersectionObserver(
      ([entry]) => setIsVisible(entry.isIntersecting),
      { threshold: 0.1 }
    );
    
    const currentRef = imgRef.current;
    currentRef && observer.observe(currentRef);

    return () => currentRef && observer.unobserve(currentRef);
  }, []);

  // Only load the image source when the component is visible
  const imgSrc = isVisible ? convertFileSrc(item.path) : null;

  if (item.id === "empty") {
    return <div className="media-item" ref={imgRef}>No media found</div>;
  }

  return (
    <div className="media-item" ref={imgRef} onClick={() => onImageClick(item)}> {/* Corrected: Pass item to onImageClick */}
      {item.type === "video" && <div className="video-indicator">▶</div>}
      {imgSrc && (
        <img
          id={item.id}
          src={imgSrc}
          alt={item.name}
          loading="lazy"
          onError={(e) => (e.target.style.display = "none")}
        />
      )}
    </div>
  );
};

export default React.memo(MediaItem, areMediaItemsEqual);