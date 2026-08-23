export interface DemoAppointment {
  date: string; // YYYY-MM-DD
  title: string;
  location: string;
}

export interface DemoProvider {
  name: string;
  specialty: string;
  initials: string;
}

export const DEMO_APPOINTMENTS: DemoAppointment[] = [
  { date: '2026-08-05', title: 'Saphnelo infusion', location: 'Oklahoma Arthritis Center' },
  { date: '2026-08-12', title: 'Rheumatology follow-up', location: 'Dr. Whitfield' },
  { date: '2026-09-02', title: 'CBC + metabolic panel', location: 'Diagnostic Lab of Oklahoma' },
];

export const DEMO_PROVIDERS: DemoProvider[] = [
  { name: 'Dr. Whitfield', specialty: 'Rheumatologist', initials: 'RW' },
  { name: 'Dr. Kim', specialty: 'Hematologist', initials: 'JK' },
];

export function upcomingCare(limit: number): DemoAppointment[] {
  return [...DEMO_APPOINTMENTS].sort((a, b) => a.date.localeCompare(b.date)).slice(0, limit);
}
