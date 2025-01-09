import type { Matter, NotificationRecord, RepeatTask, Todo, Tag } from "$src/types";
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
        // Matter 模块
        getMatter(id: string): Promise<Matter | null>;
        listMatters(): Promise<Matter[]>;
        saveMatter(matter: Matter): Promise<void>;
        deleteMatter(id: string): Promise<void>;
        queryMattersByField(field: string, value: string, exactMatch: boolean): Promise<Matter[]>;
        getMattersByRange(start: string, end: string): Promise<Matter[]>;

        // KV 模块
        setKV(key: string, value: string): Promise<void>;
        getKV(key: string): Promise<string | null>;
        deleteKV(key: string): Promise<void>;

        // Tag 模块
        createTag(names: string): Promise<void>;
        getAllTags(): Promise<Tag[]>;
        deleteTag(names: string): Promise<void>;
        updateTagLastUsedAt(names: string): Promise<void>;

        // Todo 模块
        createTodo(todo: Todo): Promise<void>;
        getTodo(id: string): Promise<Todo | null>;
        listTodos(): Promise<Todo[]>;
        updateTodo(id: string, todo: Todo): Promise<void>;
        deleteTodo(id: string): Promise<void>;

        // RepeatTask 模块
        createRepeatTask(task: RepeatTask): Promise<void>;
        getRepeatTask(id: string): Promise<RepeatTask | null>;
        listRepeatTasks(): Promise<RepeatTask[]>;
        getActiveRepeatTasks(): Promise<RepeatTask[]>;
        updateRepeatTask(id: string, task: RepeatTask): Promise<void>;
        deleteRepeatTask(id: string): Promise<void>;
        updateRepeatTaskStatus(id: string, status: number): Promise<void>;

        // Notification 模块
        getNotifications(): Promise<NotificationRecord[]>;
        saveNotification(notification: NotificationRecord): Promise<void>;
        deleteNotification(id: string): Promise<void>;
        getUnreadNotifications(): Promise<NotificationRecord[]>;
        markNotificationAsRead(id: string): Promise<void>;
        markNotificationAsReadByType(type_: number): Promise<void>;
        markAllNotificationsAsRead(): Promise<void>;
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
