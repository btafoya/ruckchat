/// Default backend URL used for local development.
/// The Web UI build injects an empty string so API calls stay on the same
/// origin as the serving server; the desktop build keeps localhost:3000.
declare const __WEB_DEFAULT_API_URL__: string | undefined;
export const DEFAULT_API_URL = __WEB_DEFAULT_API_URL__ === undefined ? 'http://localhost:3000' : __WEB_DEFAULT_API_URL__;
