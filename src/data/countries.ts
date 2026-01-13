export interface Country {
  code: string; // ISO 3166-1 alpha-2
  name: string;
  dialCode: string;
  flag: string; // Emoji flag
}

// Common countries sorted by usage, then alphabetically
export const countries: Country[] = [
  { code: "US", name: "United States", dialCode: "+1", flag: "\u{1F1FA}\u{1F1F8}" },
  { code: "GB", name: "United Kingdom", dialCode: "+44", flag: "\u{1F1EC}\u{1F1E7}" },
  { code: "IN", name: "India", dialCode: "+91", flag: "\u{1F1EE}\u{1F1F3}" },
  { code: "CA", name: "Canada", dialCode: "+1", flag: "\u{1F1E8}\u{1F1E6}" },
  { code: "AU", name: "Australia", dialCode: "+61", flag: "\u{1F1E6}\u{1F1FA}" },
  { code: "DE", name: "Germany", dialCode: "+49", flag: "\u{1F1E9}\u{1F1EA}" },
  { code: "FR", name: "France", dialCode: "+33", flag: "\u{1F1EB}\u{1F1F7}" },
  { code: "JP", name: "Japan", dialCode: "+81", flag: "\u{1F1EF}\u{1F1F5}" },
  { code: "CN", name: "China", dialCode: "+86", flag: "\u{1F1E8}\u{1F1F3}" },
  { code: "BR", name: "Brazil", dialCode: "+55", flag: "\u{1F1E7}\u{1F1F7}" },
  // Alphabetical list
  { code: "AF", name: "Afghanistan", dialCode: "+93", flag: "\u{1F1E6}\u{1F1EB}" },
  { code: "AL", name: "Albania", dialCode: "+355", flag: "\u{1F1E6}\u{1F1F1}" },
  { code: "DZ", name: "Algeria", dialCode: "+213", flag: "\u{1F1E9}\u{1F1FF}" },
  { code: "AR", name: "Argentina", dialCode: "+54", flag: "\u{1F1E6}\u{1F1F7}" },
  { code: "AT", name: "Austria", dialCode: "+43", flag: "\u{1F1E6}\u{1F1F9}" },
  { code: "BD", name: "Bangladesh", dialCode: "+880", flag: "\u{1F1E7}\u{1F1E9}" },
  { code: "BE", name: "Belgium", dialCode: "+32", flag: "\u{1F1E7}\u{1F1EA}" },
  { code: "CL", name: "Chile", dialCode: "+56", flag: "\u{1F1E8}\u{1F1F1}" },
  { code: "CO", name: "Colombia", dialCode: "+57", flag: "\u{1F1E8}\u{1F1F4}" },
  { code: "CZ", name: "Czech Republic", dialCode: "+420", flag: "\u{1F1E8}\u{1F1FF}" },
  { code: "DK", name: "Denmark", dialCode: "+45", flag: "\u{1F1E9}\u{1F1F0}" },
  { code: "EG", name: "Egypt", dialCode: "+20", flag: "\u{1F1EA}\u{1F1EC}" },
  { code: "FI", name: "Finland", dialCode: "+358", flag: "\u{1F1EB}\u{1F1EE}" },
  { code: "GR", name: "Greece", dialCode: "+30", flag: "\u{1F1EC}\u{1F1F7}" },
  { code: "HK", name: "Hong Kong", dialCode: "+852", flag: "\u{1F1ED}\u{1F1F0}" },
  { code: "HU", name: "Hungary", dialCode: "+36", flag: "\u{1F1ED}\u{1F1FA}" },
  { code: "ID", name: "Indonesia", dialCode: "+62", flag: "\u{1F1EE}\u{1F1E9}" },
  { code: "IE", name: "Ireland", dialCode: "+353", flag: "\u{1F1EE}\u{1F1EA}" },
  { code: "IL", name: "Israel", dialCode: "+972", flag: "\u{1F1EE}\u{1F1F1}" },
  { code: "IT", name: "Italy", dialCode: "+39", flag: "\u{1F1EE}\u{1F1F9}" },
  { code: "KE", name: "Kenya", dialCode: "+254", flag: "\u{1F1F0}\u{1F1EA}" },
  { code: "KR", name: "South Korea", dialCode: "+82", flag: "\u{1F1F0}\u{1F1F7}" },
  { code: "MY", name: "Malaysia", dialCode: "+60", flag: "\u{1F1F2}\u{1F1FE}" },
  { code: "MX", name: "Mexico", dialCode: "+52", flag: "\u{1F1F2}\u{1F1FD}" },
  { code: "NL", name: "Netherlands", dialCode: "+31", flag: "\u{1F1F3}\u{1F1F1}" },
  { code: "NZ", name: "New Zealand", dialCode: "+64", flag: "\u{1F1F3}\u{1F1FF}" },
  { code: "NG", name: "Nigeria", dialCode: "+234", flag: "\u{1F1F3}\u{1F1EC}" },
  { code: "NO", name: "Norway", dialCode: "+47", flag: "\u{1F1F3}\u{1F1F4}" },
  { code: "PK", name: "Pakistan", dialCode: "+92", flag: "\u{1F1F5}\u{1F1F0}" },
  { code: "PH", name: "Philippines", dialCode: "+63", flag: "\u{1F1F5}\u{1F1ED}" },
  { code: "PL", name: "Poland", dialCode: "+48", flag: "\u{1F1F5}\u{1F1F1}" },
  { code: "PT", name: "Portugal", dialCode: "+351", flag: "\u{1F1F5}\u{1F1F9}" },
  { code: "RO", name: "Romania", dialCode: "+40", flag: "\u{1F1F7}\u{1F1F4}" },
  { code: "RU", name: "Russia", dialCode: "+7", flag: "\u{1F1F7}\u{1F1FA}" },
  { code: "SA", name: "Saudi Arabia", dialCode: "+966", flag: "\u{1F1F8}\u{1F1E6}" },
  { code: "SG", name: "Singapore", dialCode: "+65", flag: "\u{1F1F8}\u{1F1EC}" },
  { code: "ZA", name: "South Africa", dialCode: "+27", flag: "\u{1F1FF}\u{1F1E6}" },
  { code: "ES", name: "Spain", dialCode: "+34", flag: "\u{1F1EA}\u{1F1F8}" },
  { code: "SE", name: "Sweden", dialCode: "+46", flag: "\u{1F1F8}\u{1F1EA}" },
  { code: "CH", name: "Switzerland", dialCode: "+41", flag: "\u{1F1E8}\u{1F1ED}" },
  { code: "TW", name: "Taiwan", dialCode: "+886", flag: "\u{1F1F9}\u{1F1FC}" },
  { code: "TH", name: "Thailand", dialCode: "+66", flag: "\u{1F1F9}\u{1F1ED}" },
  { code: "TR", name: "Turkey", dialCode: "+90", flag: "\u{1F1F9}\u{1F1F7}" },
  { code: "UA", name: "Ukraine", dialCode: "+380", flag: "\u{1F1FA}\u{1F1E6}" },
  { code: "AE", name: "United Arab Emirates", dialCode: "+971", flag: "\u{1F1E6}\u{1F1EA}" },
  { code: "VN", name: "Vietnam", dialCode: "+84", flag: "\u{1F1FB}\u{1F1F3}" },
];

export function findCountryByDialCode(dialCode: string): Country | undefined {
  return countries.find((c) => c.dialCode === dialCode);
}

export function findCountryByCode(code: string): Country | undefined {
  return countries.find((c) => c.code === code);
}
