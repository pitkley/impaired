import init, {
    getItems,
    getScores,
    hasOngoingComparison,
    nextComparison,
    pushItem,
    resetComparison,
    startComparison,
    trackResult,
} from './pkg/impaired_web.js';

const comparisonSetupModal = new bootstrap.Modal('#comparison-setup-modal');
comparisonSetupModal.show();
const comparisonSetupForm = document.getElementById("comparison-setup-form");
const comparisonSetupFormInput = document.getElementById("comparison-setup-form-input");
const comparisonSetupStart = document.getElementById("comparison-setup-start");
const comparisonLeft = document.getElementById("comparison-left");
const comparisonRight = document.getElementById("comparison-right");
const resultsContainer = document.getElementById("results-container");
const results = document.getElementById("results");
let currentComparison = null;

const populateComparisonSetupModal = (reset = false) => {
    const itemsUl = comparisonSetupModal._element.getElementsByTagName("ul")[0];
    const listInput = itemsUl.children[0];
    let items = [];
    if (!reset) {
        items = getItems()
            .map(({item}) => {
                try {
                    return JSON.parse(item).title;
                } catch (_e) {
                    return item;
                }
            })
            .map((title) => {
                const li = document.createElement("li");
                li.classList.add("list-group-item");
                li.textContent = title;
                return li;
            });
    }
    itemsUl.replaceChildren(
        listInput,
        ...items
    );
};

comparisonSetupModal._element.addEventListener('show.bs.modal', (_event) => {
    populateComparisonSetupModal(hasOngoingComparison());
});
comparisonSetupForm.onsubmit = () => {
    if (hasOngoingComparison()) {
        resetComparison();
    }

    const item = comparisonSetupFormInput.value;
    comparisonSetupFormInput.value = "";
    comparisonSetupFormInput.focus();
    if (!item) {
        // If the item is empty, we assume that the entire modal should be submitted.
        populateComparisonSetupModal(true);
        comparisonSetupStart.click();
        return false;
    }

    pushItem(item);
    populateComparisonSetupModal();

    comparisonSetupFormInput.value = "";
    comparisonSetupFormInput.focus();

    // Prevent default submit behaviour.
    return false;
};
comparisonSetupStart.addEventListener("click", () => {
    startComparison();
    setUpNextComparison();
});

comparisonLeft.addEventListener("click", (_element, _event) => {
    if (currentComparison) {
        trackResult(currentComparison.left, currentComparison.right);
    }
    setUpNextComparison();
});
comparisonRight.addEventListener("click", (_element, _event) => {
    if (currentComparison) {
        trackResult(currentComparison.right, currentComparison.left);
    }
    setUpNextComparison();
});

const parseItem = (item) => {
    let result = {};
    try {
        result = JSON.parse(item.item);
    } catch (_e) {
        result["type"] = "simple-card";
        result["title"] = item.item;
    }

    return result;
};

const CARD_GENERATOR = {
    "simple-card": (parsedItem, voteLinkHidden) => {
        const card = document.createElement("simple-card");
        if (voteLinkHidden) {
            card.hideVoteLinks();
        }
        const titleSpan = card.appendChild(document.createElement("span"));
        titleSpan.setAttribute("slot", "title");
        titleSpan.textContent = parsedItem.title;
        return card;
    },
    "ticket-card": (parsedItem, voteLinkHidden) => {
        const card = document.createElement("ticket-card");
        if (voteLinkHidden) {
            card.hideVoteLinks();
        }
        const titleSpan = card.appendChild(document.createElement("span"));
        titleSpan.setAttribute("slot", "title");
        titleSpan.textContent = parsedItem.title;
        const subtitleA = card.appendChild(document.createElement("a"));
        subtitleA.setAttribute("slot", "subtitle");
        subtitleA.setAttribute("href", parsedItem.subtitle.href);
        subtitleA.textContent = parsedItem.subtitle.name;
        const descriptionSpan = card.appendChild(document.createElement("span"));
        descriptionSpan.setAttribute("slot", "description");
        descriptionSpan.textContent = parsedItem.description;
        return card;
    },
};

const generateCardForItem = (item, voteLinkHidden = false) => {
    const parsedItem = parseItem(item);
    return CARD_GENERATOR[parsedItem.type](parsedItem, voteLinkHidden);
};

const displayResults = () => {
    // Show results container
    resultsContainer.classList.remove("d-none");

    // Create and display a card for each item, sorted by their score, in descending order (i.e. the first item is the
    // one with the most votes).
    const scores = getScores();
    const cards = scores.sort((a, b) => b.score - a.score).map(({item, _score}) => {
        const li = document.createElement("li");
        li.appendChild(generateCardForItem(item, true));
        return li;
    });
    results.replaceChildren(...cards);
}

const setUpNextComparison = () => {
    currentComparison = nextComparison();
    if (!currentComparison) {
        comparisonLeft.replaceChildren();
        comparisonRight.replaceChildren();
        displayResults();
        return;
    }

    const cardLeft = generateCardForItem(currentComparison.left);
    comparisonLeft.replaceChildren(cardLeft);
    const cardRight = generateCardForItem(currentComparison.right);
    comparisonRight.replaceChildren(cardRight);
}

const run = async () => {
    await init();
};

await run();
