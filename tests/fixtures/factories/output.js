import createFactory from './factory';
import { createEvent } from "effector";
import { createFactory2 } from './factory';
import { withFactory as _withFactory$0 } from "effector";
var _effectorFileName$0 = "/output.js";
const x = _withFactory$0({
    sid: "a4ipirl4gkpj",
    fn: ()=>createFactory(createEvent({
            sid: "29x8hthz296gt",
            loc: {
                file: _effectorFileName$0,
                line: 5,
                column: 24
            },
            name: "x"
        })),
    name: "name",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 5,
        column: 10
    }
});
const x1 = _withFactory$0({
    sid: "34b7tq0at8fpy",
    fn: ()=>createFactory(123),
    name: "x1",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 7,
        column: 11
    }
});
const x2 = _withFactory$0({
    sid: "1kozl3sawzyy9",
    fn: ()=>createFactory(456, 789),
    name: "x2",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 8,
        column: 11
    }
});
const nested1 = _withFactory$0({
    sid: "1w17o98jwlkv3",
    fn: ()=>createFactory(11, _withFactory$0({
            sid: "2mhipt4tcjzu8",
            fn: ()=>createFactory(111),
            name: "nested1",
            method: "default",
            loc: {
                file: _effectorFileName$0,
                line: 9,
                column: 34
            }
        })),
    name: "nested1",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 9,
        column: 16
    }
});
const nested2 = _withFactory$0({
    sid: "1foo588ty53em",
    fn: ()=>createFactory(22, _withFactory$0({
            sid: "1rtit3y9omj0f",
            fn: ()=>createFactory2(222),
            name: "nested2",
            method: "createFactory2#1",
            loc: {
                file: _effectorFileName$0,
                line: 10,
                column: 34
            }
        })),
    name: "nested2",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 10,
        column: 16
    }
});
const nested3 = _withFactory$0({
    sid: "21r8czhd1vtif",
    fn: ()=>createFactory(22, _withFactory$0({
            sid: "28jbrqrp31rdk",
            fn: ()=>createFactory2(222, _withFactory$0({
                    sid: "3mpga8xpw6gkz",
                    fn: ()=>createFactory(333),
                    name: "nested3",
                    method: "default",
                    loc: {
                        file: _effectorFileName$0,
                        line: 11,
                        column: 54
                    }
                })),
            name: "nested3",
            method: "createFactory2#1",
            loc: {
                file: _effectorFileName$0,
                line: 11,
                column: 34
            }
        })),
    name: "nested3",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 11,
        column: 16
    }
});
_withFactory$0({
    sid: "1ymh502w4jx07",
    fn: ()=>createFactory("no-assign"),
    name: "nested3",
    method: "default",
    loc: {
        file: _effectorFileName$0,
        line: 13,
        column: 0
    }
});