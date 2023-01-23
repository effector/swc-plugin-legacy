import { createDomain } from "effector";
var _effectorFileName$0 = "/output.js";
const domain = createDomain({
    sid: "miml62lu5xv6",
    loc: {
        file: _effectorFileName$0,
        line: 3,
        column: 15
    },
    name: "domain"
});
const someDomain = createDomain({
    sid: "11zm8yzuqbfkt",
    loc: {
        file: _effectorFileName$0,
        line: 4,
        column: 19
    },
    name: "someDomain"
});
const domainInner = someDomain.domain('inner domain', {
    sid: "ujnsx7dvtc0n",
    loc: {
        file: _effectorFileName$0,
        line: 6,
        column: 31
    },
    name: "domainInner"
});
const $store = domainInner.store(0, {
    sid: "1zim1rkmf84re",
    loc: {
        file: _effectorFileName$0,
        line: 7,
        column: 27
    },
    name: "$store"
});
