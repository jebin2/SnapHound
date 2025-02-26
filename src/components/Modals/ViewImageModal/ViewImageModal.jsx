import React, { useEffect } from "react";
import "../../../styles/Modals.css";

const ViewImageModal = ({ onClose, viewImagePath }) => {
  // Close modal when clicking outside the modal content
  const handleOutsideClick = (e) => {
    if (e.target.classList.contains("modal")) {
      onClose();
    }
  };

  // Close modal when pressing the Escape key
  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div className="modal" onClick={handleOutsideClick}>
      <div className="modal-content">
        <img src={viewImagePath} className="view-image" alt="Preview" />
      </div>
    </div>
  );
};

export default React.memo(ViewImageModal);
