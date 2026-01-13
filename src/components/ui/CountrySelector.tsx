import { useEffect, useMemo, useRef, useState } from "react";

import { countries, type Country } from "../../data/countries";

interface CountrySelectorProps {
  value: Country | null;
  onChange: (country: Country) => void;
}

// Country code badge component (since Windows doesn't render flag emojis)
function CountryBadge({ code }: { code: string }) {
  return (
    <span className="inline-flex items-center justify-center w-8 h-5 rounded text-[10px] font-bold bg-[var(--accent)]/20 text-[var(--accent)]">
      {code}
    </span>
  );
}

export function CountrySelector({ value, onChange }: CountrySelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Filter countries based on search
  const filteredCountries = useMemo(() => {
    if (!search.trim()) return countries;
    const query = search.toLowerCase();
    return countries.filter(
      (c) =>
        c.name.toLowerCase().includes(query) ||
        c.dialCode.includes(query) ||
        c.code.toLowerCase().includes(query)
    );
  }, [search]);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setSearch("");
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Focus search input when dropdown opens
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setIsOpen(false);
      setSearch("");
    } else if (e.key === "Enter" && filteredCountries.length > 0) {
      onChange(filteredCountries[0]);
      setIsOpen(false);
      setSearch("");
    }
  };

  const handleSelect = (country: Country) => {
    onChange(country);
    setIsOpen(false);
    setSearch("");
  };

  return (
    <div ref={containerRef} className="relative">
      {/* Trigger Button */}
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2.5 px-3 py-2 bg-[var(--bg-secondary)] rounded-lg border border-transparent hover:border-[var(--border-light)] transition-colors h-[38px]"
      >
        {value ? (
          <>
            <CountryBadge code={value.code} />
            <span className="text-sm font-mono text-[var(--text-primary)]">
              {value.dialCode}
            </span>
          </>
        ) : (
          <span className="text-sm text-[var(--text-secondary)]">
            Select
          </span>
        )}
        <svg
          className={`w-3.5 h-3.5 text-[var(--text-secondary)] transition-transform ${isOpen ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {/* Dropdown */}
      {isOpen && (
        <div className="absolute top-full left-0 mt-1.5 w-80 bg-[var(--bg-primary)] border border-[var(--border-light)] rounded-xl shadow-2xl z-50 overflow-hidden">
          {/* Search Input */}
          <div className="p-3 border-b border-[var(--border-light)]">
            <div className="relative">
              <svg
                className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-secondary)]"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <input
                ref={searchInputRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Search country or code..."
                className="w-full pl-10 pr-3 py-2.5 bg-[var(--bg-secondary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] rounded-lg outline-none text-sm border border-transparent focus:border-[var(--accent)]"
              />
            </div>
          </div>

          {/* Country List */}
          <div
            ref={listRef}
            className="overflow-y-auto max-h-48 overscroll-contain"
          >
            {filteredCountries.length === 0 ? (
              <div className="px-4 py-10 text-center text-sm text-[var(--text-secondary)]">
                No countries found
              </div>
            ) : (
              filteredCountries.map((country) => (
                <button
                  key={country.code}
                  type="button"
                  onClick={() => handleSelect(country)}
                  className={`w-full flex items-center gap-3 px-4 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left ${value?.code === country.code ? "bg-[var(--bg-hover)]" : ""
                    }`}
                >
                  <CountryBadge code={country.code} />
                  <span className="flex-1 text-sm text-[var(--text-primary)]">
                    {country.name}
                  </span>
                  <span className="text-sm text-[var(--text-secondary)] font-mono tabular-nums">
                    {country.dialCode}
                  </span>
                  {value?.code === country.code && (
                    <svg
                      className="w-4 h-4 text-[var(--accent)] ml-1"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
