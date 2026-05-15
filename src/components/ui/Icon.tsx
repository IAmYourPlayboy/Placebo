import * as Icons from "../Icons";
import type { SVGProps } from "react";

type IconsModule = typeof Icons;
export type IconName = {
  [K in keyof IconsModule]: IconsModule[K] extends (props: SVGProps<SVGSVGElement>) => JSX.Element ? K : never;
}[keyof IconsModule];

type Props = SVGProps<SVGSVGElement> & {
  name: IconName;
  size?: number;
};

/**
 * Tiny by-name wrapper around the SVG icon components in ../Icons.
 * Only the SF-stroke icons added in M1 (HomeIcon, GridIcon, ...) are
 * type-compatible with the SVGProps signature; legacy `Ic*` icons take
 * different props and remain available via direct import.
 */
export function Icon({ name, size = 20, ...rest }: Props) {
  const Component = Icons[name] as (p: SVGProps<SVGSVGElement>) => JSX.Element;
  return <Component width={size} height={size} {...rest} />;
}
