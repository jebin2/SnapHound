/**
 * Deduplicate media items by ID
 * @param {Array} prevItems - Previous media items
 * @param {Array} newItems - New media items to add
 * @returns {Array} Combined and deduplicated array
 */
export const deduplicateMediaItems = (prevItems, newItems) => {
	return [...new Map([...prevItems, ...newItems].map(item => [item.id, item])).values()];
  };
  
  /**
   * Check if media items should update based on changes
   * @param {Object} prev - Previous item
   * @param {Object} next - Next item
   * @returns {boolean} Whether the items are equal
   */
  export const areMediaItemsEqual = (prev, next) => {
	return (
	  prev.item.id === next.item.id &&
	  prev.item.path === next.item.path &&
	  prev.item.type === next.item.type &&
	  prev.item.name === next.item.name
	);
  };