import { mediaUrl } from '../../lib/format';
import { fallbackImage } from '../../lib/cart';
import type { ProductImageModel } from '../../types';

interface Props {
  image?: ProductImageModel | null;
  alt?: string;
  sizes?: string;
  className?: string;
  loading?: 'lazy' | 'eager';
  decoding?: 'async' | 'sync' | 'auto';
}

/**
 * Responsive product image: AVIF source when available, JPEG/PNG fallback,
 * explicit dimensions (avoids CLS) and lazy loading by default.
 */
export default function ProductImage({
  image,
  alt,
  sizes,
  className,
  loading = 'lazy',
  decoding = 'async',
}: Props) {
  const src = mediaUrl(image?.url) || fallbackImage;
  const hasSrcset = Boolean(image?.srcset || image?.avif_srcset);
  const responsiveSizes = sizes ?? '100vw';

  const img = (
    <img
      src={src}
      srcSet={image?.srcset ?? undefined}
      sizes={hasSrcset ? responsiveSizes : undefined}
      alt={alt ?? image?.alt ?? ''}
      width={image?.width ?? undefined}
      height={image?.height ?? undefined}
      loading={loading}
      decoding={decoding}
      className={className}
    />
  );

  if (image?.avif_srcset) {
    return (
      <picture style={{ display: 'contents' }}>
        <source type="image/avif" srcSet={image.avif_srcset} sizes={responsiveSizes} />
        {img}
      </picture>
    );
  }
  return img;
}
