import { IconStar, IconStarFilled } from '@tabler/icons-react';

export default function Stars({ value = 0, size = 14 }: { value?: number; size?: number }) {
  const rounded = Math.round(value);
  return (
    <span style={{ display: 'inline-flex', gap: 1, color: '#f59e0b' }} aria-label={`${value} out of 5`}>
      {[1, 2, 3, 4, 5].map((star) =>
        star <= rounded ? <IconStarFilled key={star} size={size} /> : <IconStar key={star} size={size} />,
      )}
    </span>
  );
}
