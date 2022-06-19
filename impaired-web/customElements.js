class StyleInheritingHTMLElement extends HTMLElement {
    constructor() {
        super();

        const shadowRoot = this.attachShadow({mode: 'open'});
        const stylesheetLinks = document.getElementsByTagName('link');
        for (const stylesheetLink of stylesheetLinks) {
            shadowRoot.appendChild(stylesheetLink.cloneNode(true));
        }
    }
}

class SimpleCardHTMLElement extends StyleInheritingHTMLElement {
    constructor() {
        super();

        const template = document.getElementById('simple-card');
        const templateContent = template.content;
        const templateNode = templateContent.cloneNode(true);
        this.voteLinks = templateNode.querySelectorAll(".vote-link");
        this.shadowRoot.appendChild(templateNode);
    }

    hideVoteLinks() {
        this.voteLinks.forEach((node) => node.style.display = 'none');
    }
}

class TicketCardHTMLElement extends StyleInheritingHTMLElement {
    constructor() {
        super();

        const template = document.getElementById('ticket-card');
        const templateContent = template.content;
        const templateNode = templateContent.cloneNode(true);
        this.voteLinks = templateNode.querySelectorAll(".vote-link");
        this.shadowRoot.appendChild(templateNode);
    }

    hideVoteLinks() {
        this.voteLinks.forEach((node) => node.style.display = 'none');
    }
}

customElements.define('simple-card', SimpleCardHTMLElement);
customElements.define('ticket-card', TicketCardHTMLElement);
