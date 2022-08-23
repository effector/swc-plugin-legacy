import { createEffect, createEvent, sample } from "effector";
const event = createEvent({
    sid: "36r1dlh80h4ib",
    name: "event"
});
const s = sample({
    and: [
        {
            source: event,
            target: createEffect(()=>{}, {
                sid: "wlg6l6xh1141",
                name: "target"
            })
        }
    ],
    or: {
        sid: "15chlc7yr0j10",
        name: "s"
    }
});
