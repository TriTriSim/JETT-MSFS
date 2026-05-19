// SimConnect unit strings
export const units = {
  KNOTS: 'knots',
  FEET: 'feet',
  FEET_PER_MINUTE: 'feet per minute',
  METERS: 'meters',
  METERS_PER_SECOND: 'meters per second',
  DEGREES: 'degrees',
  RADIANS: 'radians',
  MACH: 'mach',
  PERCENT: 'percent',
  PERCENT_OVER_100: 'percent over 100',
  NUMBER: 'number',
  BOOL: 'bool',
  POUNDS: 'pounds',
  GALLONS: 'gallons',
  RPM: 'rpm',
  CELSIUS: 'celsius',
  FAHRENHEIT: 'fahrenheit',
  PASCAL: 'pascal',
  HOURS: 'hours',
  MINUTES: 'minutes',
  SECONDS: 'seconds',
} as const;

export type UnitName = typeof units[keyof typeof units];
