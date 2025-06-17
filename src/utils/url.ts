export function handleServiceUrl(str: string) {
    try {
        if (!str) {
            return { rpc: "", authorization: "" }
        }
        let service = str;
        if (service.endsWith('/')) {
            service = str.slice(0, -1); // remove trailing slash
        }
        let scheme = service.split('://')[0];
        let token = ""
        let domain = ""
        let parts = service.split('://')[1].split('@');
        if (parts.length === 2) {
            token = parts[0];
            domain = parts[1];
        } else {
            domain = parts[0];
        }
        let rpc = `${scheme}://${domain}`;
        let authorization = `Bearer ${token || ''}`
        return { rpc, authorization }
    } catch (error) {
        return { rpc: "", authorization: "" }
    }
}