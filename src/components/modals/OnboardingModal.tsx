import { useState } from "react";

import { countries, type Country } from "../../data/countries";
import { userService } from "../../services";
import { useUserStore } from "../../store/userStore";
import { CountrySelector } from "../ui/CountrySelector";

interface OnboardingModalProps {
  onComplete: () => void;
}

const MIN_PHONE_DIGITS = 7;
const MAX_PHONE_DIGITS = 15;

// Get digits only from phone string
function getDigitsOnly(phone: string): string {
  return phone.replace(/\D/g, "");
}

// Client-side validation for the local number part
function validateLocalNumber(localNumber: string): string | null {
  const digits = getDigitsOnly(localNumber);

  if (digits.length === 0) {
    return null; // Empty is ok, will be caught on submit
  }

  // Local number typically 4-12 digits depending on country
  if (digits.length < 4) {
    return "Enter at least 4 digits";
  }

  if (digits.length > 12) {
    return "Number is too long for this country";
  }

  return null;
}

// Validate the full phone number (dial code + local)
function validateFullPhone(dialCode: string, localNumber: string): string | null {
  const localDigits = getDigitsOnly(localNumber);
  const dialDigits = getDigitsOnly(dialCode);
  const totalDigits = dialDigits.length + localDigits.length;

  if (localDigits.length === 0) {
    return null; // Empty is ok, will be caught on submit
  }

  if (totalDigits < MIN_PHONE_DIGITS) {
    return `Phone number needs ${MIN_PHONE_DIGITS - totalDigits} more digit${MIN_PHONE_DIGITS - totalDigits > 1 ? "s" : ""}`;
  }

  if (totalDigits > MAX_PHONE_DIGITS) {
    return "Phone number exceeds maximum length";
  }

  return null;
}

// Map backend errors to user-friendly messages
function friendlyError(err: string): string {
  const msg = String(err).toLowerCase();
  if (msg.includes("already registered")) {
    return "This phone number is already registered.";
  }
  if (msg.includes("too short")) {
    return "Phone number too short";
  }
  if (msg.includes("too long")) {
    return "Phone number too long";
  }
  if (msg.includes("empty")) {
    return "Phone number is required";
  }
  if (msg.includes("only contain digits")) {
    return "Only digits allowed";
  }
  return String(err);
}

// Default to India
const defaultCountry = countries.find((c) => c.code === "IN") || countries[0];

export function OnboardingModal({ onComplete }: OnboardingModalProps) {
  const currentUser = useUserStore((state) => state.currentUser);
  const loadCurrentUser = useUserStore((state) => state.loadCurrentUser);

  const [selectedCountry, setSelectedCountry] = useState<Country>(defaultCountry);
  const [localNumber, setLocalNumber] = useState("");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  const localDigits = getDigitsOnly(localNumber);
  const dialDigits = getDigitsOnly(selectedCountry.dialCode);
  const totalDigits = dialDigits.length + localDigits.length;

  const localError = localNumber.trim() ? validateLocalNumber(localNumber) : null;
  const fullError = localNumber.trim()
    ? validateFullPhone(selectedCountry.dialCode, localNumber)
    : null;

  const isValid = localDigits.length >= 4 && !localError && !fullError;

  const handleSave = async () => {
    if (!localNumber.trim()) {
      setError("Please enter your phone number");
      return;
    }

    if (localError || fullError) {
      setError(localError || fullError || "Invalid phone number");
      return;
    }

    setSaving(true);
    setError("");

    try {
      // Combine dial code and local number
      const fullPhone = `${selectedCountry.dialCode}${localDigits}`;
      await userService.setPhoneNumber(fullPhone);
      await loadCurrentUser();
      onComplete();
    } catch (err) {
      setError(friendlyError(String(err)));
    } finally {
      setSaving(false);
    }
  };

  // Format phone for display
  const formatPhoneDisplay = () => {
    if (!localDigits) return "";
    return `${selectedCountry.dialCode} ${localNumber}`;
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center" style={{ top: 'var(--titlebar-height)' }}>
      <div className="absolute inset-0 bg-black/60" />

      <div className="relative w-[480px] max-w-[92vw] rounded-2xl bg-[var(--bg-primary)] border border-[var(--border-light)] shadow-2xl">
        {/* Header */}
        <div className="px-6 pt-6 pb-4">
          <h2 className="text-xl font-semibold text-[var(--text-primary)]">
            Enter your phone number
          </h2>
          <p className="mt-2 text-sm text-[var(--text-secondary)]">
            Your phone number is your unique Pulse ID. Others will use it to
            connect with you.
          </p>
        </div>

        {/* Form */}
        <div className="px-6 pb-6">
          <label className="text-xs text-[var(--accent)] block mb-2">
            Phone Number
          </label>

          {/* Country + Phone Input Row */}
          <div className="flex gap-2">
            <CountrySelector
              value={selectedCountry}
              onChange={(country) => {
                setSelectedCountry(country);
                setError("");
              }}
            />

            <div className="flex-1 relative">
              <input
                type="tel"
                value={localNumber}
                onChange={(e) => {
                  // Only allow digits, spaces, and dashes for formatting
                  const value = e.target.value.replace(/[^\d\s-]/g, "");
                  setLocalNumber(value);
                  setError("");
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && isValid) {
                    handleSave();
                  }
                }}
                placeholder="Phone number"
                className="w-full px-3 py-2 bg-[var(--bg-secondary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] rounded-lg outline-none text-sm font-mono border border-transparent focus:border-[var(--accent)]"
                autoFocus
              />
            </div>
          </div>

          {/* Info row */}
          <div className="flex justify-between items-center mt-2">
            <p className="text-xs text-[var(--text-secondary)]">
              {localDigits ? (
                <span className="font-mono">{formatPhoneDisplay()}</span>
              ) : (
                "Enter your number without country code"
              )}
            </p>
            <p
              className={`text-xs tabular-nums ${
                totalDigits > 0 && totalDigits < MIN_PHONE_DIGITS
                  ? "text-red-400"
                  : "text-[var(--text-secondary)]"
              }`}
            >
              {totalDigits}/{MAX_PHONE_DIGITS}
            </p>
          </div>

          {/* Error */}
          {(error || (localDigits.length > 0 && (localError || fullError))) && (
            <p className="mt-2 text-xs text-red-500">
              {error || localError || fullError}
            </p>
          )}

          {/* Current ID */}
          <div className="mt-4 rounded-lg bg-[var(--bg-secondary)] px-3 py-2">
            <p className="text-xs text-[var(--text-secondary)]">Current ID</p>
            <p className="text-xs text-[var(--text-primary)] font-mono truncate">
              {currentUser?.id || "Loading..."}
            </p>
          </div>

          {/* Submit Button */}
          <button
            onClick={handleSave}
            disabled={saving || !isValid}
            className="w-full mt-5 px-3 py-2.5 text-sm font-medium bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {saving ? "Saving..." : "Continue with this number"}
          </button>
        </div>
      </div>
    </div>
  );
}
