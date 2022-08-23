import {createEffect, Event, sample} from "effector";

export function factory<T>(event: Event<T>) {
    return sample({
        source: event,
        target: createEffect(console.log)
    });
}