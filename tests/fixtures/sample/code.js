import {createEffect, createEvent, sample} from "effector";

const event = createEvent();

const s = sample({
    source: event,
    target: createEffect(() => {})
})