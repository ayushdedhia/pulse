import { useEffect, useRef, type RefObject } from "react";

type Dimension = "height" | "width" | "both";

interface UseCSSVariableOptions {
  /** CSS variable name (without --) */
  variable: string;
  /** Which dimension to track */
  dimension?: Dimension;
  /** Update on window resize */
  updateOnResize?: boolean;
}

/**
 * Hook to dynamically set CSS variables based on element dimensions.
 *
 * @example
 * // Track height
 * const ref = useCSSVariable({ variable: 'titlebar-height' });
 * return <div ref={ref}>...</div>
 *
 * @example
 * // Track width
 * const ref = useCSSVariable({ variable: 'sidebar-width', dimension: 'width' });
 *
 * @example
 * // Track both
 * const ref = useCSSVariable({ variable: 'card', dimension: 'both' });
 * // Sets --card-height and --card-width
 */
export function useCSSVariable<T extends HTMLElement = HTMLDivElement>({
  variable,
  dimension = "height",
  updateOnResize = true,
}: UseCSSVariableOptions): RefObject<T> {
  const ref = useRef<T>(null);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    const updateVariable = () => {
      const root = document.documentElement;

      if (dimension === "height" || dimension === "both") {
        const height = element.offsetHeight;
        const heightVar = dimension === "both" ? `--${variable}-height` : `--${variable}`;
        root.style.setProperty(heightVar, `${height}px`);
      }

      if (dimension === "width" || dimension === "both") {
        const width = element.offsetWidth;
        const widthVar = dimension === "both" ? `--${variable}-width` : `--${variable}`;
        root.style.setProperty(widthVar, `${width}px`);
      }
    };

    updateVariable();

    if (updateOnResize) {
      window.addEventListener("resize", updateVariable);
      return () => window.removeEventListener("resize", updateVariable);
    }
  }, [variable, dimension, updateOnResize]);

  return ref;
}
