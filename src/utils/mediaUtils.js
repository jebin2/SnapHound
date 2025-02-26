/**
 * Deduplicate media items by ID
 * @param {Array} prevItems - Previous media items
 * @param {Array} newItems - New media items to add
 * @returns {Array} Combined and deduplicated array
 */
export const deduplicateMediaItems = (prevItems, newItems) => {
	const seen = new Map();

	// Add existing items to the map
	prevItems.forEach(item => {
		seen.set(`${item.id}-${item.path}`, item);
	});

	// Add new items only if they are not already in the map
	newItems.forEach(item => {
		const key = `${item.id}-${item.path}`;
		if (!seen.has(key)) {
			seen.set(key, item);
		}
	});

	return Array.from(seen.values());
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