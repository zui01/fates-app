<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import { EXCLUDE_HOLIDAYS_BIT, generateDescription } from "$lib/utils/repeatTime";
    import { t } from "svelte-i18n";

    export let value: string;
    export let onUpdateValue: (newValue: string) => void;

    let weekdaysBits: number;
    let startTime: string;
    let endTime: string;
    let excludeHolidays: boolean;
    let lastBits: number;

    const WEEKDAY_LABELS = [
        $t("app.repeat.repeatTime.weekdays.sun"),
        $t("app.repeat.repeatTime.weekdays.mon"),
        $t("app.repeat.repeatTime.weekdays.tue"),
        $t("app.repeat.repeatTime.weekdays.wed"),
        $t("app.repeat.repeatTime.weekdays.thu"),
        $t("app.repeat.repeatTime.weekdays.fri"),
        $t("app.repeat.repeatTime.weekdays.sat"),
    ];

    function initializeValues() {
        const [bits, start, end] = value.split("|");
        weekdaysBits = parseInt(bits);
        startTime = start.trim();
        endTime = end.trim();
        excludeHolidays = !!(weekdaysBits & EXCLUDE_HOLIDAYS_BIT);
        lastBits = weekdaysBits;
    }

    initializeValues();

    const toggleDay = (dayIndex: number) => {
        const bit = 1 << dayIndex;
        let newBits = weekdaysBits & bit ? weekdaysBits & ~bit : weekdaysBits | bit;
        if (newBits == 0) {
            return;
        }
        weekdaysBits = newBits;
        updateValue();
    };

    function isValidTimeRange(start: string, end: string): boolean {
        if (!start || !end) return false;

        try {
            // use fixed date to create Date object, only compare time part
            const startDate = new Date(`2000-01-01T${start}`);
            const endDate = new Date(`2000-01-01T${end}`);

            // check if it is a valid Date object
            if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
                return false;
            }

            // convert to minutes for comparison, avoid milliseconds error
            const startMinutes = startDate.getHours() * 60 + startDate.getMinutes();
            const endMinutes = endDate.getHours() * 60 + endDate.getMinutes();

            return startMinutes < endMinutes;
        } catch (error) {
            console.error("time format validation error:", error);
            return false;
        }
    }

    function updateValue() {
        const newBits = (weekdaysBits & ~EXCLUDE_HOLIDAYS_BIT) | (excludeHolidays ? EXCLUDE_HOLIDAYS_BIT : 0);
        value = `${newBits}|${startTime}|${endTime}`;
        console.log("repeatTimeSelector new value", value);
        onUpdateValue(value);
        lastBits = newBits;
    }

    function handleSwitchChange(checked: boolean) {
        excludeHolidays = checked;
        updateValue();
    }

    $: description = generateDescription(lastBits);

    function handleStartTimeChange(event: Event) {
        const input = event.target as HTMLInputElement;
        startTime = input.value;
        validateAndUpdateTime();
    }

    function handleEndTimeChange(event: Event) {
        const input = event.target as HTMLInputElement;
        endTime = input.value;
        validateAndUpdateTime();
    }

    function validateAndUpdateTime() {
        // if (startTime && endTime) {
        //     if (!isValidTimeRange(startTime, endTime)) {
        //         const startDate = new Date(`2000-01-01T${startTime}`);
        //         startDate.setHours(startDate.getHours() + 1);
        //         const newEndTime = startDate.toTimeString().slice(0, 5);
        //         endTime = newEndTime;
        //     }
        //     updateValue();
        // }
    }
</script>

<div class="w-[300px] rounded-lg border bg-white p-4 shadow-sm">
    <div class="space-y-4">
        <div class="flex flex-col gap-1">
            <Label class="text-xl font-bold">{$t("app.repeat.repeatTime.title")}</Label>
            <Label class="text-sm text-gray-500">{description.split("|").join(" ")}</Label>
        </div>

        <div class="flex items-center gap-4">
            <Input
                type="time"
                value={startTime}
                onchange={handleStartTimeChange}
                class="w-[48px] bg-background font-bold border-none p-0 shadow-none focus-visible:ring-0 focus-visible:ring-offset-0"
            />
            <span class="text-gray-500">{$t("app.repeat.repeatTime.to")}</span>
            <Input
                type="time"
                value={endTime}
                onchange={handleEndTimeChange}
                class="w-[48px] bg-background font-bold border-none p-0 shadow-none focus-visible:ring-0 focus-visible:ring-offset-0"
            />
        </div>

        <div class="flex gap-1">
            {#each WEEKDAY_LABELS as label, index}
                <button
                    class="h-8 w-8 rounded-full text-sm {weekdaysBits & (1 << index)
                        ? 'bg-blue-500 text-[#fff]'
                        : 'bg-gray-100'}"
                    on:click={() => toggleDay(index)}
                >
                    {label}
                </button>
            {/each}
        </div>

        <div class="flex items-center justify-between">
            <Label for="exclude-holidays" class="font-bold">{$t("app.repeat.repeatTime.excludeHolidays")}</Label>
            <Switch id="exclude-holidays" checked={excludeHolidays} onCheckedChange={handleSwitchChange} />
        </div>
    </div>
</div>
