interface P { active?: boolean; size?: number; color?: string }
const C = (a?: boolean) => a ? "var(--accent)" : "var(--t3)";

export function IcHome({ active, size = 24 }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M3 9.5L12 3L21 9.5V20a1 1 0 01-1 1H5a1 1 0 01-1-1V9.5z"
        stroke={C(active)} strokeWidth={active ? 2.1 : 1.75} strokeLinejoin="round"
        fill={active ? "rgba(232,52,90,.12)" : "none"} />
      <path d="M9 21V12h6v9" stroke={C(active)} strokeWidth={active ? 2.1 : 1.75} strokeLinecap="round" />
    </svg>
  );
}

export function IcGrid({ active, size = 24 }: P) {
  const c = C(active); const sw = active ? 2.1 : 1.75;
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <rect x="3"    y="3"    width="7.5" height="7.5" rx="1.5" stroke={c} strokeWidth={sw} />
      <rect x="13.5" y="3"    width="7.5" height="7.5" rx="1.5" stroke={c} strokeWidth={sw} />
      <rect x="3"    y="13.5" width="7.5" height="7.5" rx="1.5" stroke={c} strokeWidth={sw} />
      <rect x="13.5" y="13.5" width="7.5" height="7.5" rx="1.5" stroke={c} strokeWidth={sw} />
    </svg>
  );
}

export function IcPlus({ size = 22 }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 22 22" fill="none">
      <path d="M11 3v16M3 11h16" stroke="#fff" strokeWidth={2.5} strokeLinecap="round" />
    </svg>
  );
}

export function IcUsers({ active, size = 24 }: P) {
  const c = C(active); const sw = active ? 2.1 : 1.75;
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="9" cy="7" r="3.5" stroke={c} strokeWidth={sw} />
      <path d="M2 20c0-3.866 3.134-7 7-7s7 3.134 7 7" stroke={c} strokeWidth={sw} strokeLinecap="round" />
      <path d="M16 3.5a3.5 3.5 0 010 7M22 20c0-3.314-2.239-6.1-5.333-6.8"
        stroke={c} strokeWidth={sw} strokeLinecap="round" />
    </svg>
  );
}

export function IcUser({ active, size = 24 }: P) {
  const c = C(active); const sw = active ? 2.1 : 1.75;
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="7.5" r="4" stroke={c} strokeWidth={sw} />
      <path d="M3 20c0-4.418 4.03-8 9-8s9 3.582 9 8"
        stroke={c} strokeWidth={sw} strokeLinecap="round" />
    </svg>
  );
}

export function IcBell({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0"
        stroke={color} strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function IcClock({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="9" stroke={color} strokeWidth={1.75} />
      <path d="M12 7v5l3 3" stroke={color} strokeWidth={1.75} strokeLinecap="round" />
    </svg>
  );
}

export function IcBookmark({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M6 3h12a1 1 0 011 1v17l-7-4-7 4V4a1 1 0 011-1z"
        stroke={color} strokeWidth={1.75} strokeLinejoin="round" />
    </svg>
  );
}

export function IcSearch({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="11" cy="11" r="7" stroke={color} strokeWidth={1.75} />
      <path d="M16.5 16.5L21 21" stroke={color} strokeWidth={1.75} strokeLinecap="round" />
    </svg>
  );
}

export function IcMenu({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M4 6h16M4 12h16M4 18h10"
        stroke={color} strokeWidth={1.75} strokeLinecap="round" />
    </svg>
  );
}

export function IcHeart({ size = 14, filled = false }: { size?: number; filled?: boolean }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill={filled ? "var(--accent)" : "none"}>
      <path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"
        stroke={filled ? "var(--accent)" : "var(--t3)"} strokeWidth={1.75} strokeLinejoin="round" />
    </svg>
  );
}

export function IcComment({ size = 13 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"
        stroke="var(--t3)" strokeWidth={1.75} strokeLinejoin="round" />
    </svg>
  );
}

export function IcShare({ size = 13 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M22 2L11 13" stroke="var(--t3)" strokeWidth={1.75} strokeLinecap="round" />
      <path d="M22 2L15 22l-4-9-9-4 20-7z"
        stroke="var(--t3)" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function IcPin({ size = 10 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M21 10c0 7-9 13-9 13S3 17 3 10a9 9 0 1118 0z"
        stroke="var(--t3)" strokeWidth={2} />
      <circle cx="12" cy="10" r="3" stroke="var(--t3)" strokeWidth={2} />
    </svg>
  );
}

// Apple-style "..." horizontal more
export function IcMoreH({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="5"  cy="12" r="1.5" fill={color}/>
      <circle cx="12" cy="12" r="1.5" fill={color}/>
      <circle cx="19" cy="12" r="1.5" fill={color}/>
    </svg>
  );
}

// Verified badge (Apple/Instagram style checkmark in circle)
export function IcVerified({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="var(--accent)"/>
      <path d="M7.5 12l3 3 6-6" stroke="#fff" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
}

// Chat bubble (Apple Messages style)
export function IcChat({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"
        stroke={color} strokeWidth={1.75} strokeLinejoin="round"/>
    </svg>
  );
}

// Apple SF-style person.badge.plus (add friend)
export function IcPersonAdd({ size = 22, color = "var(--t1)" }: P) {
  const sw = 1.75;
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <circle cx="9" cy="7" r="4" stroke={color} strokeWidth={sw}/>
      <path d="M2 20c0-4 3.1-7 7-7" stroke={color} strokeWidth={sw} strokeLinecap="round"/>
      <path d="M17 13v6M14 16h6" stroke={color} strokeWidth={sw} strokeLinecap="round"/>
    </svg>
  );
}

// Film / movie camera (Apple style)
export function IcFilm({ size = 22, color = "var(--t1)" }: P) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <rect x="2" y="6" width="20" height="12" rx="2" stroke={color} strokeWidth={1.75}/>
      <path d="M6 6V4M10 6V4M14 6V4M18 6V4M6 20v-2M10 20v-2M14 20v-2M18 20v-2"
        stroke={color} strokeWidth={1.75} strokeLinecap="round"/>
      <path d="M2 10h20M2 14h20" stroke={color} strokeWidth={1.5}/>
    </svg>
  );
}

// =====================================================================
// SF-style stroke icons used by the shell (sidebar, topbar, tabbar).
// All use currentColor + stroke 1.5, no fill. Sized via width/height props.
// Older `Ic*` icons above are kept for legacy screens until M4-M6 rewrites.
// =====================================================================

type SvgProps = React.SVGProps<SVGSVGElement>;

const SF_BASE: SvgProps = {
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.5,
  strokeLinecap: "round",
  strokeLinejoin: "round",
};

export function HomeIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M3 10l9-7 9 7v11a1 1 0 0 1-1 1h-5v-7h-6v7H4a1 1 0 0 1-1-1V10z"/>
    </svg>
  );
}

export function GridIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <rect x="3" y="3" width="7.5" height="7.5" rx="1.5"/>
      <rect x="13.5" y="3" width="7.5" height="7.5" rx="1.5"/>
      <rect x="3" y="13.5" width="7.5" height="7.5" rx="1.5"/>
      <rect x="13.5" y="13.5" width="7.5" height="7.5" rx="1.5"/>
    </svg>
  );
}

export function PlusIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M12 5v14M5 12h14"/>
    </svg>
  );
}

export function PlusSmallIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M12 6v12M6 12h12"/>
    </svg>
  );
}

export function UsersIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="9" cy="7" r="3.5"/>
      <path d="M2 20c0-3.866 3.134-7 7-7s7 3.134 7 7"/>
      <path d="M16 3.5a3.5 3.5 0 0 1 0 7M22 20c0-3.314-2.239-6.1-5.333-6.8"/>
    </svg>
  );
}

export function UserIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="12" cy="7.5" r="4"/>
      <path d="M3 20c0-4.418 4.03-8 9-8s9 3.582 9 8"/>
    </svg>
  );
}

export function BellIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
      <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
    </svg>
  );
}

export function ClockIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="12" cy="12" r="9"/>
      <path d="M12 7v5l3 3"/>
    </svg>
  );
}

export function StarIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M12 3l2.9 6 6.6.95-4.75 4.62 1.12 6.55L12 17.97l-5.87 3.15 1.12-6.55L2.5 9.95 9.1 9z"/>
    </svg>
  );
}

export function FolderIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z"/>
    </svg>
  );
}

export function GearIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>
  );
}

export function ArrowLeftIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M15 18l-6-6 6-6"/>
    </svg>
  );
}

export function ArrowRightIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M9 6l6 6-6 6"/>
    </svg>
  );
}

export function RefreshIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M3 12a9 9 0 0 1 15.5-6.3L21 8"/>
      <path d="M21 3v5h-5"/>
      <path d="M21 12a9 9 0 0 1-15.5 6.3L3 16"/>
      <path d="M3 21v-5h5"/>
    </svg>
  );
}

export function SearchIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="11" cy="11" r="7"/>
      <path d="M16.5 16.5L21 21"/>
    </svg>
  );
}

export function MoonIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/>
    </svg>
  );
}

export function SunIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="12" cy="12" r="4"/>
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
    </svg>
  );
}

// Toggle ("auto" / system theme): a light-half + dark-half circle.
export function ToggleIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <circle cx="12" cy="12" r="9"/>
      <path d="M12 3a9 9 0 0 0 0 18z" fill="currentColor" stroke="none"/>
    </svg>
  );
}

export function CloseIcon(props: SvgProps) {
  return (
    <svg {...SF_BASE} {...props}>
      <path d="M6 6l12 12M18 6L6 18"/>
    </svg>
  );
}
