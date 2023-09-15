import { createContext, useContext, useEffect, useState } from "react";
import { OnTauriEvent } from "../utils";
import { RustError, ScraperState } from "../types";
type LiveScraperContextProps = ScraperState & {

}
type LiveScraperContextProviderProps = {
  children: React.ReactNode;
}

export const LiveScraperContext = createContext<LiveScraperContextProps>({
  is_running: false,
  last_run: null,
  error: null,
});

export const useLiveScraperContext = () => useContext(LiveScraperContext);

export const LiveScraperContextProvider = ({ children }: LiveScraperContextProviderProps) => {
  const [is_running, setIsRunning] = useState(false);
  const [last_run, setLastRun] = useState<Date | null>(null);
  const [error, setError] = useState<RustError | null>(null);

  useEffect(() => {
    OnTauriEvent("LiveScraper:Start", () => {
      setLastRun(new Date())
    });
    OnTauriEvent("LiveScraper:Error", (error: RustError) => {
      setIsRunning(false)
      setError(error)
    });
    return () => { }
  }, []);

  return (
    <LiveScraperContext.Provider value={{ is_running, last_run, error }}>
      {children}
    </LiveScraperContext.Provider>
  )
}