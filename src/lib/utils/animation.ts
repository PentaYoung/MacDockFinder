export function calcMagnification(
  iconIndex: number,
  iconCount: number,
  mousePos: number,
  iconSize: number,
  spacing: number
): number {
  const center = mousePos;
  const iconCenter = iconIndex * (iconSize + spacing) + iconSize / 2;
  const distance = Math.abs(iconCenter - center);
  const maxDist = (iconCount * (iconSize + spacing)) / 2;
  const normalized = Math.max(0, 1 - distance / maxDist);
  return 1.0 + normalized * 0.3 * (1 - Math.pow(1 - normalized, 3));
}
