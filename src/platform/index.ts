import type { Matter, NotificationRecord } from "$src/types";
export const REFRESH_TIME_PROGRESS = "refresh-time-progress";

export type UnlistenFn = () => void;

interface Event<T> {
    event: string;
    id: number;
    payload: T;
}

export interface PlatformAPI {
    event: {
        emit(event: string, payload?: unknown): Promise<void>;
        listen<T>(event: string, handler: (event: Event<T>) => void, options?: any): Promise<UnlistenFn>;
    };

    dailyProgressBar: {
        initialize(): Promise<void>;
        destroy(): Promise<void>;
    };

    clipboard: {
        writeText(text: string): Promise<void>;
    };

    notification: {
        show(title: string, body: string, options?: any): Promise<void>;
        requestPermission(): Promise<"default" | "denied" | "granted">;
        isPermissionGranted(): Promise<boolean>;
        sendNotification(title: string, body: string): Promise<void>;
    };

    getVersion(): Promise<string>;

    autostart?: {
        enable(): Promise<void>;
        disable(): Promise<void>;
        isEnabled(): Promise<boolean>;
    };



    storage: {
        getMatter(id: string): Promise<Matter | null>;
        listMatters(): Promise<Matter[]>;
        saveMatter(matter: Matter): Promise<void>;
        deleteMatter(id: string): Promise<void>;

        getNotifications(): Promise<NotificationRecord[]>;
        saveNotification(notification: NotificationRecord): Promise<void>;
        deleteNotification(id: string): Promise<void>;
    };

    window: {
        minimize(): Promise<void>;
        maximize(): Promise<void>;
        close(): Promise<void>;
        show(): Promise<void>;
        hide(): Promise<void>;
    };

    tray?: {
        create(options: any): Promise<void>;
        destroy(): Promise<void>;
        setMenu(menu: any): Promise<void>;
    };

    updater?: {
        checkForUpdates(): Promise<{ hasUpdate: boolean; version?: string }>;
        downloadAndInstall(): Promise<void>;
    };
}

// @ts-ignore
export const isTauri = typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;
export const isWeb = !isTauri;

// 获取当前平台的实现
export async function getPlatform(): Promise<PlatformAPI> {
    if (isTauri) {
        console.log("Platform is Tauri !!!");
        const { default: tauriPlatform } = await import("./tauri");
        return tauriPlatform;
    } else {
        console.log("Platform is Web !!!");
        const { default: webPlatform } = await import("./web");
        return webPlatform;
    }
}

let platformInstance: PlatformAPI | null = null;

export async function initializePlatform(): Promise<PlatformAPI> {
    if (!platformInstance) {
        platformInstance = await getPlatform();
    }
    return platformInstance;
}

export default {
    get instance() {
        if (!platformInstance) {
            throw new Error("Platform not initialized. Call initializePlatform() first.");
        }
        return platformInstance;
    },
};
