[
  /* -------------------------------------------------------- */
  /*                         Math Mode                        */
  /* -------------------------------------------------------- */
  { trigger: 'mk', replacement: '$$0$', options: 'tA' },
  { trigger: 'dm', replacement: '$$\n$0\n$$', options: 'tAw' },
  { trigger: 'beg', replacement: '\\begin{$0}\n$1\n\\end{$0}', options: 'mA' },

  // Dashes
  // {trigger: "--", replacement: "–", options: "tA"},
  // {trigger: "–-", replacement: "—", options: "tA"},
  // {trigger: "—-", replacement: "---", options: "tA"},

  /* -------------------------------------------------------- */
  /*                     Visual Operations                    */
  /* -------------------------------------------------------- */
  {
    trigger: 'U',
    replacement: '\\underbrace{ ${VISUAL} }_{ $0 }',
    options: 'mA',
  },
  {
    trigger: 'O',
    replacement: '\\overbrace{ ${VISUAL} }^{ $0 }',
    options: 'mA',
  },

  { trigger: 'B', replacement: '\\underset{ $0 }{ ${VISUAL} }', options: 'mA' },

  { trigger: 'K', replacement: '\\cancelto{ $0 }{ ${VISUAL} }', options: 'mA' },
  { trigger: 'C', replacement: '\\cancel{ ${VISUAL} }', options: 'mA' },
  { trigger: 'Q', replacement: '\\boxed{ ${VISUAL} }', options: 'mA' },
  { trigger: 'S', replacement: '\\sqrt{ ${VISUAL} }', options: 'mA' },

  /* -------------------------------------------------------- */
  /*                   Equations & Alignment                  */
  /* -------------------------------------------------------- */
  // Simple replacements
  { trigger: '==', replacement: '&=', options: 'mA' },
  { trigger: 'tag', replacement: '\\tag{${0:1}}$1', options: 'MA' },
  {
    trigger: /(qquad|quad)/,
    replacement: '\\[[0]] $0',
    options: 'rMA',
    description: 'Insert spacing commands (\\quad or \\qquad)',
  },

  /* -------------------------------------------------------- */
  /*                   Environment Wrappers                   */
  /* -------------------------------------------------------- */
  {
    trigger: /^(cases|align|gather)/,
    /**
     * Replaces a matched keyword with a LaTeX environment.
     * @param {Array} match - The match array where the second element
     *                        contains the LaTeX environment name.
     * @returns {string} A string formatted with the LaTeX \begin and \end
     *                   commands, using the extracted environment name.
     */
    replacement: (match) => {
      const env_name = match[1];
      return `\\begin{${env_name}}\n$0\n\\end{${env_name}}`;
    },
    options: 'rmA',
    description: 'Wrap selection in LaTeX cases, align, or gather environments',
  },

  /* ---------------- Selection-Based: Gather --------------- */
  {
    trigger: 'G',
    /**
     * Replaces newline characters in the selected text with LaTeX line
     * breaks and wraps the text in a LaTeX gather environment.
     * @param {string} sel - The selected text to be formatted.
     * @returns {string} The formatted text wrapped in a LaTeX gather
     *                   environment.
     */
    replacement: (sel) => {
      const processed = sel.replace(/\n/g, ' \\\\\n');
      return `\\begin{gather}\n${processed}\n\\end{gather}`;
    },
    options: 'mvA',
    description: 'Format selected lines into a gather environment',
  },

  /* ---------------- Selection-Based: Align ---------------- */
  // Convert lines to align environment and add &=
  {
    trigger: 'A',
    replacement: (sel) => {
      let processed = sel.replace(/=/g, '&=');
      processed = processed.replace(/\n/g, ' \\\\\n');
      return `\\begin{align}\n${processed}\n\\end{align}`;
    },
    options: 'mvA',
    description: 'Format selection as aligned equations with &=',
  },

  // Inline formatting of dollar math with auto-aligned operators
  {
    trigger: 'A',
    replacement: (sel) => {
      const config = {
        operators: {
          basic: ['=', '>', '<'],
          latex: [
            '\\geq',
            '\\iff',
            '\\impliedby',
            '\\implies',
            '\\in',
            '\\leq',
            '\\neq',
          ],
          special: ['\\int', '\\prod', '\\sum'],
        },
        templates: {
          align: (content) =>
            `$$\n\\begin{align}\n${content}\n\\end{align}\n$$`,
        },
      };

      const utils = {
        getAllOperators() {
          return [...config.operators.basic, ...config.operators.latex];
        },
        createOperatorPattern() {
          const operators = this.getAllOperators().map((op) =>
            op.replace(/\\/g, '\\\\')
          );
          return new RegExp(`(?:\\$\\$[^$]+\\$\\$|\\$[^$]+\\$)`, 'g');
        },
        isWithinSpecialOperator(str, pos) {
          return config.operators.special.some((op) => {
            const opIndex = str.lastIndexOf(op, pos);
            if (opIndex === -1) return false;
            let bracket_count = 0;
            for (let i = opIndex; i < str.length; i++) {
              if (str[i] === '{') bracket_count++;
              if (str[i] === '}') bracket_count--;
              if (bracket_count === 0 && i >= pos) return true;
            }
            return false;
          });
        },
        formatEquation(equation) {
          equation = equation.replace(/^\$\$?\s*|\s*\$\$?$/g, '');
          const lines = equation.split(/(?:\\\\)?\s*\n\s*|\s*\\\\\s*/);
          return lines.map((line) => {
            let processed = line.trim();
            let first_op = null;
            let first_op_index = Infinity;
            for (const op of this.getAllOperators()) {
              const index = processed.indexOf(op);
              if (index !== -1 && index < first_op_index) {
                first_op_index = index;
                first_op = op;
              }
            }
            if (
              first_op &&
              !this.isWithinSpecialOperator(processed, first_op_index)
            ) {
              processed = processed.replace(first_op, `&${first_op}`);
            }
            return processed;
          });
        },
      };

      const processEquations = (sel) => {
        const pattern = utils.createOperatorPattern();
        const matches = sel.match(pattern);
        if (!matches) return null;
        const all_equations = matches.flatMap((eq) => utils.formatEquation(eq));
        const formatted = all_equations
          .filter((eq) => eq.trim())
          .map(
            (eq, index, arr) => `${eq}${index < arr.length - 1 ? ' \\\\' : ''}`
          )
          .join('\n');
        return config.templates.align(formatted);
      };

      return processEquations(sel);
    },
    options: 'tvA',
    description:
      'Auto-align math blocks in dollar-delimited equations using &= insertion',
  },

  /* -------------------------------------------------------- */
  /*               Text Environment & Formatting              */
  /* -------------------------------------------------------- */

  // Font commands: \mathbf, \mathcal, \mathrm
  {
    trigger: /(bf|cal|rm)/,
    replacement: (match) => {
      const command_key = match[1]; // 'bf', 'cal', or 'rm'
      return `\\math${command_key}{$0}$1`;
    },
    options: 'rmA',
    description:
      'Insert bold (\\mathbf), caligraghic (\\mathcal), or roman upright (\\mathrm) formatting for math symbols',
  },

  // Common math operator formatting: \mathrm{Re}, \mathrm{Im}, \mathrm{Tr}
  {
    trigger: /(Re|Im|trace)/,
    replacement: (match) => {
      const content = match[1] === 'trace' ? 'Tr' : match[1];
      return `\\mathrm{${content}}`;
    },
    options: 'rmA',
    description:
      'Format standard operators in upright roman: \\mathrm{Re}, \\mathrm{Im}, and \\mathrm{Tr}',
  },

  // Apply \mathbf to bolden a character followed by ,\. or \.,
  {
    trigger: /([a-zA-Z])(,\\.|\\.,)/,
    replacement: '\\mathbf{[[0]]}',
    options: 'rmA',
  },

  /* ------------------ Math Number Systems ----------------- */
  // CC -> \mathbb{C} (ℂ)
  // FF -> \mathbb{F} (𝔽)
  // II -> \mathbb{I} (𝕀)
  // NN -> \mathbb{N} (ℕ)
  // QQ -> \mathbb{Q} (ℚ)
  // RR -> \mathbb{R} (ℝ)
  // ZZ -> \mathbb{Z} (ℤ)
  // LL -> \mathcal{L} (ℒ)
  // HH -> \mathcal{H} (ℋ)
  {
    trigger: /(CC|FF|II|NN|QQ|RR|ZZ|LL|HH)/,
    replacement: (match) => {
      const double_letter = match[1];
      const letter = double_letter[0];
      const bb_letters = ['C', 'F', 'I', 'N', 'Q', 'R', 'Z'];
      const is_mathbb = bb_letters.includes(letter);
      const command = is_mathbb ? 'mathbb' : 'mathcal';
      return `\\${command}{${letter}}\$0`;
    },
    options: 'rmA',
    description:
      'Insert \\mathbb (ℂ, ℝ, ℕ, etc.) or \\mathcal (ℒ, ℋ) notation for standard sets and operators',
  },

  /* ----------------- Inline Text Insertion ---------------- */
  { trigger: 'text', replacement: '\\text{$0}$1', options: 'mA' },
  { trigger: '"', replacement: '\\text{$0}$1', options: 'mA' },

  // Logic words as spaced text: AND / OR
  {
    trigger: /(AND|OR)/,
    replacement: (match) => {
      const connective = match[1].toLowerCase();
      return `\\quad \\text{${connective}} \\quad`;
    },
    options: 'rmA',
    description:
      'Insert logic connective as text with spacing (\\text{and}, \\text{or})',
  },

  /* -------------------------------------------------------- */
  /*                     Basic Operations                     */
  /* -------------------------------------------------------- */

  // Fractions
  { trigger: '//', replacement: '\\frac{$0}{$1}$2', options: 'mA' },

  // nth Roots (e.g., 3sq → \sqrt[3]{...}, sq → \sqrt{...})
  {
    trigger: /([0-9nmk]*)sq/,
    replacement: (match) => {
      const root_index = match[1];
      return root_index === '' ? '\\sqrt{$0}$1' : `\\sqrt[${root_index}]{$0}`;
    },
    options: 'rmA',
    description:
      'Insert a square root (sq → \\sqrt{}) or indexed root (e.g., 3sq → \\sqrt[3]{})',
  },

  // Natural exponent
  { trigger: 'ee', replacement: 'e^{$0}$1', options: 'mA' },

  // Functions: det, exp, log, ln
  {
    trigger: /([^\\])(det|exp|log|ln)/,
    replacement: '[[0]]\\[[1]]',
    options: 'rmA',
    description: 'Insert backslash before common math functions',
  },

  /* -------------------------------------------------------- */
  /*                        Superscript                       */
  /* -------------------------------------------------------- */

  /* -------------- Generic Superscript Trigger ------------- */
  { trigger: '^', replacement: '^{$0}$1', options: 'mA' },

  /* ----------- Explicit Superscript With Tabstop ---------- */
  { trigger: 'sp', replacement: '^{$0}', options: 'mA' },

  /* ------------------- Text Superscript ------------------- */
  { trigger: 'stp', replacement: '^\\text{$0}', options: 'mA' },

  // e.g., 5th → 5^{\text{th}}
  {
    trigger: /([A-Za-z])(th)/,
    replacement: (match) => `${match[1]}^{\\text{${match[2]}}}\$0`,
    options: 'rmA',
    description: 'Auto text superscript (e.g., nth → n^{\\text{th}})',
  },

  // General superscripts (pw, ^, rd)
  {
    trigger: /^(pw|\^|rd)$/,
    replacement: '^{$0}$1',
    options: 'rmA',
    description: 'Insert exponent with tabstop (pw, ^, rd → ^{...})',
  },

  // Common shorthand superscripts (e.g., sr → ^{2}, invs → ^{-1})
  {
    trigger: /(sr|cb|invs|conj|\+\+|--)/,
    replacement: (match) => {
      const superscript_map = {
        sr: '2', // square
        cb: '3', // cube
        invs: '-1', // inverse
        conj: '*', // conjugate
        '++': '+', // superscript plus
        '--': '-', // superscript minus
      };
      return `^{${superscript_map[match[1]]}}`;
    },
    options: 'rmA',
    description: 'Common superscripts: sr, cb, invs, conj, ++, -- → ^{...}',
  },

  // Automatic wrapping of multi-char superscripts (e.g., ^xy → ^{xy})
  {
    trigger: /\^([a-zA-Z0-9]{2,})/,
    replacement: (match) => `^{${match[1]}}\$0`,
    options: 'rmA',
  },

  /* -------------------------------------------------------- */
  /*                         Subscript                        */
  /* -------------------------------------------------------- */

  /* --------------- Generic Subscript Trigger -------------- */
  { trigger: '_', replacement: '_{$0}$1', options: 'mA' },

  /* ------------ Explicit Subscript With Tabstop ----------- */
  { trigger: 'sc', replacement: '_{$0}', options: 'mA' },

  /* -------------------- Text Subscript -------------------- */
  { trigger: 'sts', replacement: '_\\text{$0}', options: 'mA' },

  // Common patterns like a1 → a_{1} or a_2 → a_{2}
  {
    // Match letter or {group} followed by 1–2 digits, but not if preceded by _
    trigger: /(?<!_)({[a-zA-Z]+}|[a-zA-Z])_?(\d{1,2})/,
    replacement: '[[0]]_{[[1]]}', // Output: base_{digits}
    options: 'rmA',
    description:
      'Auto subscript: a1 or {x}1 → a_{1}, {x}_{1}; skips if already subscripted',
    priority: -1,
  },

  /* ---------- Shorthand Repeated-Index Subscripts --------- */
  {
    trigger: /([A-Za-hj-z])(ii|jj|kk|mm|nn|pp)/,
    replacement: (match) => `${match[1]}_{${match[2][1]}}\$0`,
    options: 'rmA',
    description: 'Auto letter subscript (e.g., ann → a_{n})',
  },

  /* ----------------- Subscript With Offset ---------------- */
  {
    trigger: /([A-Za-z])(\w)(p|s)(\d)/,
    replacement: (match) => {
      const operator = match[3] === 'p' ? '+' : '-';
      return `${match[1]}_{${match[2]} ${operator} ${match[4]}}\$0`;
    },
    options: 'rmA',
    description: 'Offset subscripts: anp1 → a_{n + 1}, ans1 → a_{n - 1}',
  },

  // Automatic wrapping of multi-char subscripts (e.g., _xy → _{xy})
  {
    trigger: /_([a-zA-Z0-9]{2,})/,
    replacement: (match) => `_{${match[1]}}\$0`,
    options: 'rmA',
  },

  /* -------------------------------------------------------- */
  /*                       Greek Letters                      */
  /* -------------------------------------------------------- */
  {
    trigger: /([@:])([a-gik-pr-uxzDFGLOPSTUX])/,
    replacement: (match) => {
      const prefix = match[1]; // '@' for regular, ':' for variant
      const greek_letter_key = match[2];

      const greek_map = {
        a: 'alpha', // α
        b: 'beta', // β
        g: 'gamma', // γ
        G: 'Gamma', // Γ
        d: 'delta', // δ
        D: 'Delta', // Δ
        e: 'epsilon', // ϵ (variant: ε)
        z: 'zeta', // ζ
        t: 'theta', // θ (variant: ϑ)
        T: 'Theta', // Θ
        i: 'iota', // ι
        k: 'kappa', // κ
        l: 'lambda', // λ
        L: 'Lambda', // Λ
        m: 'mu', // μ
        n: 'nu', // ν
        x: 'xi', // ξ
        X: 'Xi', // Ξ
        r: 'rho', // ρ (variant: ϱ)
        s: 'sigma', // σ
        S: 'Sigma', // Σ
        u: 'upsilon', // υ
        U: 'Upsilon', // ϒ
        f: 'phi', // φ (variant: ϕ)
        F: 'Phi', // Φ
        p: 'psi', // ψ
        P: 'Psi', // Ψ
        c: 'chi', // χ
        o: 'omega', // ω
        O: 'Omega', // Ω
      };

      // Letters that support \var variants in LaTeX
      const variant_letters = ['e', 't', 'r', 'f'];

      const base_command = greek_map[greek_letter_key];
      const is_variant =
        prefix === ':' && variant_letters.includes(greek_letter_key);

      const latex_command = is_variant
        ? `\\var${base_command}`
        : `\\${base_command}`;

      return `${latex_command}\$0`;
    },
    options: 'rmA',
    description:
      'Insert Greek letter using @ (e.g., @a → \\alpha) or variant with : (e.g., :f → \\varphi → ϕ)',
  },

  // Snippet variables can be used as shortcuts when writing
  // snippets. For example, ${GREEK} below is shorthand for
  // "alpha|beta|gamma|Gamma|delta|..." You can edit snippet
  // variables under the Advanced snippet settings section.

  {
    trigger: /\\\\(${GREEK})(,\\.|\\.,)/,
    replacement: '\\boldsymbol{\\[[0]]}',
    options: 'rmA',
  },
  {
    trigger: /([^\\\\])(${GREEK}|${SYMBOL})/,
    replacement: '[[0]]\\[[1]]',
    options: 'rmA',
    description: 'Add backslash before Greek letters and symbols',
  },
  {
    trigger: /\\\\(${GREEK}|${SYMBOL}|${MORE_SYMBOLS})([A-Za-z])/,
    replacement: '\\[[0]] [[1]]',
    options: 'rmA',
    description: 'Insert space after Greek letters and symbols',
  },
  {
    trigger: /\\\\(${GREEK}|${SYMBOL}) sr/,
    replacement: '\\[[0]]^{2}',
    options: 'rmA',
  },
  {
    trigger: /\\\\(${GREEK}|${SYMBOL}) cb/,
    replacement: '\\[[0]]^{3}',
    options: 'rmA',
  },
  {
    trigger: /\\\\(${GREEK}|${SYMBOL}) rd/,
    replacement: '\\[[0]]^{$0}$1',
    options: 'rmA',
  },
  {
    trigger: /\\\\(${GREEK}|${SYMBOL}) (bar|dot|hat|tilde|und|vec)/,
    replacement: (match) => {
      const symbol = match[2] == 'und' ? 'underline' : match[2];
      return `\\${symbol}{\\${match[1]}}\$0`;
    },
    options: 'rmA',
  },

  /* -------------------------------------------------------- */
  /*            Text Accents - Over And Underlining           */
  /* -------------------------------------------------------- */
  {
    trigger: /([A-Za-z]?)(hat|bar|tilde|und|vec)/,
    replacement: (match) => {
      const letter = match[1]; // may be empty
      const accent_key = match[2];
      const accent_command = accent_key === 'und' ? 'underline' : accent_key;

      return letter
        ? `\\${accent_command}{${letter}}\$0` // Case: Ahat → \hat{\A}
        : `\\${accent_command}{\$0}\$1`; // Case: hat → \hat{...}
    },
    options: 'rmA',
    description:
      'Apply accent to a letter (e.g., ahat → \\hat{\\a}) or use as wrapper (e.g., hat → \\hat{...})',
  },
  {
    trigger: /,(b|h|H|t|T|v|V)/,
    replacement: (match) => {
      const line_type_map = {
        b: 'bar',
        h: 'hat',
        H: 'widehat',
        t: 'tilde',
        T: 'widetilde',
        v: 'vec',
        V: 'overrightarrow',
      };

      const line_type = line_type_map[match[1]];
      return `\\${line_type}{\$0}\$1`;
    },
    options: 'rmA',
    description:
      'Applies bar, hat, tilde, vector, or wide/over variants to a symbol',
  },

  /* -------------------------------------------------------- */
  /*                           Dots                           */
  /* -------------------------------------------------------- */
  {
    trigger: /([A-Za-z]?)(dot|ddot)/,
    replacement: (match) => {
      const letter = match[1]; // may be empty
      const accent = match[2]; // "dot" or "ddot"

      return letter
        ? `\\${accent}{${letter}}\$0` // e.g., xdot → \dot{x}
        : `\\${accent}{\$0}\$1`; // e.g., dot → \dot{...}
    },
    options: 'rmA',
    description:
      'Add dot or double dot accent. Use alone (dot → \\dot{...}) or with a letter (xdot → \\dot{x})',
    priority: -1,
  },
  { trigger: '...', replacement: '\\dots', options: 'mA' },
  { trigger: ',d', replacement: '\\dot{$0}$1', options: 'mA' },
  { trigger: ',D', replacement: '\\ddot{$0}$1', options: 'mA', priority: 2 },

  /* ---------------------- Center Dot ---------------------- */
  { trigger: '**', replacement: '\\cdot$0', options: 'nA' }, // ⋅, Inline
  { trigger: '*', replacement: '\\cdot$0', options: 'MA' }, // ⋅, Block
  { trigger: 'cdot', replacement: '\\cdot', options: 'mw', priority: 0 }, // ⋅
  { trigger: 'ccdot', replacement: '\\cdots', options: 'mw', priority: 0 }, // ⋯

  /* -------------------------------------------------------- */
  /*                 Symbols & Math Operators                 */
  /* -------------------------------------------------------- */
  {
    trigger: /(ooo|\+\-|\-\+|xx|nabl|del|para)/,
    replacement: (match) => {
      const symbol_map = {
        ooo: '\\infty', // ∞
        '+-': '\\pm', // ±
        '-+': '\\mp', // ∓
        xx: '\\times', // ×
        nabl: '\\nabla', // ∇
        del: '\\nabla', // ∇
        para: '\\parallel', // ∥
      };
      return symbol_map[match[1]];
    },
    options: 'rmA',
    description: 'Common math symbols: ±, ∞, ×, ∇, ∥, etc.',
  },

  /* -------------------------------------------------------- */
  /*                   Comparison Operators                   */
  /* -------------------------------------------------------- */
  {
    trigger: /(===|!=|>=|<=|!>=|!<=|!<|!>|nle|nge|>>|<<|simm|sim=|simeq|prop)/,
    replacement: (match) => {
      const raw = match[1];
      const is_not = raw.startsWith('!');

      const cmp_map = {
        '===': '\\equiv', // ≡
        '!=': '\\neq', // ≠
        '>=': '\\geq', // ≥
        '<=': '\\leq', // ≤
        '>>': '\\gg', // ≫
        '<<': '\\ll', // ≪
        simeq: '\\simeq', // ≃
        'sim=': '\\simeq', // ≃
        simm: '\\sim', // ∼
        prop: '\\propto', // ∝

        // Conditional negations
        '!<': '\\not<', // ≮
        '!>': '\\not>', // ≯
        '!<=': '\\not\\leq', // ≰
        '!>=': '\\not\\geq', // ≱
        nle: '\\not\\leq', // ≰
        nge: '\\not\\geq', // ≱
      };

      return cmp_map[raw];
    },
    options: 'rmA',
    description: 'Comparison operators: ≠, ≥, ≰, ≱, ≃, ∼, ∝, etc.',
  },

  /* -------------------------------------------------------- */
  /*                          Arrows                          */
  /* -------------------------------------------------------- */
  {
    trigger: /(<=>|<->|->|!->|nto|!>|=>|=<)/,
    replacement: (match) => {
      const arrow_map = {
        '<=>': '\\Leftrightarrow', // ⇔
        '<->': '\\leftrightarrow', // ↔
        '->': '\\to', // →
        '!->': '\\not\\to', // ↛
        nto: '\\not\\to', // ↛
        '!>': '\\mapsto', // ↦
        '=>': '\\implies', // ⟹
        '=<': '\\impliedby', // ⟸
      };
      return arrow_map[match[1]];
    },
    options: 'rmA',
    description: 'Logical and mapping arrows: →, ⇒, ↛, ↔, etc.',
  },

  /* -------------------------------------------------------- */
  /*                         Brackets                         */
  /* -------------------------------------------------------- */
  { trigger: 'ang', replacement: '\\langle $0 \\rangle $1', options: 'mA' },
  { trigger: 'mod', replacement: '|$0|$1', options: 'mA' },

  /* ------------------------- Norm ------------------------- */
  {
    trigger: /(n|N)orm/,
    replacement: (match) => {
      const style = match[1] === 'n' ? 'vert' : 'Vert';
      return `\\l${style} $0 \\r${style} $1`;
    },
    options: 'rmA',
    description: 'Norm notation: \\lvert ... \\rvert or \\lVert ... \\rVert',
  },

  /* --------------- Floor & Ceiling Notation --------------- */
  {
    trigger: /(lr)?(ceil|floor)/,
    replacement: (match) => {
      const big = match[1];
      const op = match[2];
      let left = `\\l${op}`,
        right = `\\r${op}`;
      if (big) {
        left = `\\left ${left}`;
        right = `\\right ${right}`;
      }
      return `${left} $0 ${right}$1`;
    },
    options: 'rmA',
    description: 'Floor and ceiling, optionally with \\left/\\right',
  },

  /* ----------------- Left-Right Enclosures ---------------- */
  {
    trigger: /(lr)?([\(\[\{])/,
    replacement: (match) => {
      const big = match[1]; // 'lr' if \left/\right is requested, else undefined
      const left = match[2]; // opening delimiter: (, [, or {

      // Define matching closing delimiters
      const right_map = {
        '(': ')',
        '[': ']',
        '{': '}',
      };

      // get the matching closing delimiter
      const right = right_map[left];

      // Wrap delimiter with \left/\right if needed
      const wrap = (symbol, side) => (big ? `\\${side}${symbol} ` : symbol);

      const left_tex = wrap(left, 'left'); // final left delimiter
      const right_tex = wrap(right, 'right'); // final right delimiter

      // Return the full expression with tabstops
      return `${left_tex}$0${right_tex}$1`;
    },
    options: 'rmA',
    description:
      'Output LaTeX enclosures: (), [], {}, optionally with \\left/\\right.',
  },
  { trigger: 'lr|', replacement: '\\left| $0 \\right| $1', options: 'mA' },
  { trigger: 'lra', replacement: '\\left< $0 \\right> $1', options: 'mA' },

  /* ------------ Left-Right Selection Enclosures ----------- */
  { trigger: '(', replacement: '(${VISUAL})', options: 'mA' },
  { trigger: '[', replacement: '[${VISUAL}]', options: 'mA' },
  { trigger: '{', replacement: '{${VISUAL}}', options: 'mA' },
  { trigger: '|', replacement: '|${VISUAL}|', options: 'mA' },

  /* -------------------------------------------------------- */
  /*                           Logic                          */
  /* -------------------------------------------------------- */
  { trigger: 'lneg', replacement: '\\neg', options: 'mA' }, // ¬
  { trigger: 'landd', replacement: '\\land', options: 'mA' }, // ∧
  { trigger: 'lorr', replacement: '\\lor', options: 'mA' }, // ∨
  { trigger: 'lsome', replacement: '\\exists', options: 'mA' }, // ∃
  { trigger: 'lnone', replacement: '\\nexists', options: 'mA' }, // ∄
  { trigger: 'lall', replacement: '\\forall', options: 'mA' }, // ∀
  { trigger: 'lnotall', replacement: '\\not \\forall', options: 'mA' }, // ∀

  /* -------------------------------------------------------- */
  /*                        Set Theory                        */
  /* -------------------------------------------------------- */
  { trigger: 'sand', replacement: '\\cap', options: 'mA' }, // ∩
  { trigger: 'capp', replacement: '\\cap', options: 'mA' }, // ∩
  { trigger: 'sor', replacement: '\\cup', options: 'mA' }, // ∪
  { trigger: 'cupp', replacement: '\\cup', options: 'mA' }, // ∪
  { trigger: 'inn', replacement: '\\in', options: 'mA' }, // ∈
  { trigger: 'ninn', replacement: '\\notin', options: 'mA' }, // ∉
  { trigger: 'notin', replacement: '\\notin', options: 'mA' }, // ∉
  { trigger: 'sless', replacement: '\\setminus', options: 'mA' },
  { trigger: 'setmin', replacement: '\\setminus', options: 'mA' },
  { trigger: '\\\\\\', replacement: '\\setminus', options: 'mA' },
  { trigger: 'subeq', replacement: '\\subseteq', options: 'mA' }, // ⊆
  { trigger: 'sub=', replacement: '\\subseteq', options: 'mA' }, // ⊆
  { trigger: 'supeq', replacement: '\\supseteq', options: 'mA' }, // ⊇
  { trigger: 'sup=', replacement: '\\supseteq', options: 'mA' }, // ⊇
  { trigger: 'eset', replacement: '\\emptyset', options: 'mA' }, // ∅
  { trigger: 'sempty', replacement: '\\emptyset', options: 'mA' }, // ∅
  { trigger: 'set', replacement: '\\{ $0 \\}$1', options: 'mWA', priority: -1 },
  {
    trigger: /b(cap|cup)/,
    replacement: '\\big[[0]]_{${0:i} \\in ${1:I}}^{${2:N}} $3',
    options: 'rmA',
    description: 'Output big union and big intersection',
  },

  /* -------------------------------------------------------- */
  /*                     Sequence Notation                    */
  /* -------------------------------------------------------- */
  {
    trigger: /(r)?seq/,
    replacement: (match) => {
      const sub_r = match[1];
      const sequence_notation =
        '{\\langle ${0:x}${1:_}{${2:n}} \\rangle}_{${2:n} = ${3:1}}^{${4:\\infty}}';
      return sub_r
        ? sequence_notation + ' \\subseteq \\mathbb{R}'
        : sequence_notation;
    },
    options: 'rmA',
    description:
      'Output sequence notation with angle brackets. Option for defining number set.',
  },

  /* -------------------------------------------------------- */
  /*                 Sigma Summation Notation                 */
  /* -------------------------------------------------------- */
  //{ trigger: 'sum', replacement: '\\sum', options: 'mA' },
  {
    trigger: /(f|n|s|st)?sum/,
    replacement: (match) => {
      const sum_type = match[1]; // 'f', 'n', 's', 'st', or undefined
      const default_notation = '\\sum_{${0:k} = ${1:1}}^{${2:n}}';
      const templates = {
        f: [default_notation, '${3:f}(${0:k})', '$5'].join(' '),
        n: [
          '\\sum_{${0:i = 1}}^{${1:m}}',
          '\\sum_{${2:j = 1}}^{${3:n}}',
          '$4',
        ].join(' '),
        s: [default_notation, '${3:x}${4:_}{${0:k}}', '$5'].join(' '),
        st: '\\sum_{${0:s} \\in ${1:C}} $2',
      };
      return templates[sum_type] ?? default_notation;
    },
    options: 'rmA',
    description:
      'Sigma notation: plain, nested, function, set-indexed, or sequence summations.',
  },

  /* -------------------------------------------------------- */
  /*                     Product Notation                     */
  /* -------------------------------------------------------- */
  //{ trigger: 'prod', replacement: '\\prod', options: 'mA' },
  {
    trigger: 'prod',
    replacement: '\\prod_{${0:i} = ${1:1}}^{${2:N}} $3',
    options: 'mA',
  },

  /* -------------------------------------------------------- */
  /*                      Limit Notation                      */
  /* -------------------------------------------------------- */

  /* ---------------------- Math Block ---------------------- */
  {
    trigger: /(f|s)?lim/,
    replacement: (match) => {
      const lim_type = match[1]; // 'f', 's', or undefined
      const default_notation = '\\lim_{${0:n} \\to ${1:\\infty}}';
      const templates = {
        f: '\\lim_{${0:x} \\to ${1:c}} ${2:f}(${0:x}) $3',
        s: [default_notation, '${2:x}${3:_}{${0:n}}', '$4'].join(' '),
      };
      return templates[lim_type] ?? [default_notation, '$2'].join(' ');
    },
    options: 'rMA',
    description: 'Limit notation: plain, function, or sequence notations.',
  },
  {
    trigger: 'lim',
    replacement: '\\lim_{${0:n} \\to ${1:\\infty} } $2',
    options: 'MA',
  },
  {
    trigger: /lm(inf|sup)/,
    replacement: '\\lim[[0]]_{${0:n} \\to ${1:\\infty}} $2',
    options: 'rMA',
    description: 'Limit Superior and Inferior',
  },

  /* ---------------------- Inline Math --------------------- */
  {
    trigger: /(f|s)?lim/,
    replacement: (match) => {
      const lim_type = match[1]; // 'f', 's', or undefined
      const default_notation = '\\lim\\limits__{${0:n} \\to ${1:\\infty}}';
      const templates = {
        f: '\\lim\\limits__{${0:x} \\to ${1:c}} ${2:f}(${0:x}) $3',
        s: [default_notation, '${2:x}${3:_}{${0:n}}', '$4'].join(' '),
      };
      return templates[lim_type] ?? [default_notation, '$2'].join(' ');
    },
    options: 'rnA',
    description: 'Limit notation: plain, function, or sequence notations.',
  },
  {
    trigger: 'lim',
    replacement: '\\lim\\limits_{${0:n} \\to ${1:\\infty}} $2',
    options: 'nA',
  },
  {
    trigger: /lm(inf|sup)/,
    replacement: '\\lim[[0]]\\limits_{${0:n} \\to ${1:\\infty}} $2',
    options: 'rnA',
    description: 'Limit Superior and Inferior',
  },

  /* -------------------------------------------------------- */
  /*                  Derivatives & Integrals                 */
  /* -------------------------------------------------------- */
  {
    trigger: 'par',
    replacement: '\\frac{\\partial ${0:y}}{\\partial ${1:x}} $2',
    options: 'm',
  },
  {
    trigger: /pa([A-Za-z])([A-Za-z])/,
    replacement: '\\frac{ \\partial [[0]] }{ \\partial [[1]] } ',
    options: 'rm',
  },
  { trigger: 'ddt', replacement: '\\frac{d}{dt} ', options: 'mA' },

  {
    trigger: /([^\\])int/,
    replacement: '[[0]]\\int',
    options: 'mA',
    priority: -1,
  },
  { trigger: '\\int', replacement: '\\int $0 \\, d${1:x} $2', options: 'm' },
  {
    trigger: 'dint',
    replacement: '\\int_{${0:0}}^{${1:1}} $2 \\, d${3:x} $4',
    options: 'mA',
  },
  {
    trigger: /(o|i|ii)int/,
    replacement: '\\[[0]]int',
    options: 'rmA',
  },
  {
    trigger: 'oinf',
    replacement: '\\int_{0}^{\\infty} $0 \\, d${1:x} $2',
    options: 'mA',
  },
  {
    trigger: 'infi',
    replacement: '\\int_{-\\infty}^{\\infty} $0 \\, d${1:x} $2',
    options: 'mA',
  },

  /* -------------------------------------------------------- */
  /*                     Taylor Expansions                    */
  /* -------------------------------------------------------- */
  {
    trigger: 'tayl',
    replacement:
      "${0:f}(${1:x} + ${2:h}) = ${0:f}(${1:x}) + ${0:f}'(${1:x})${2:h} + ${0:f}''(${1:x}) \\frac{${2:h}^{2}}{2!} + \\dots$3",
    options: 'mA',
    description: 'Taylor expansion',
  },

  /* -------------------------------------------------------- */
  /*                       Trigonometry                       */
  /* -------------------------------------------------------- */
  {
    trigger: /([^\\])(arcsin|sin|arccos|cos|arctan|tan|csc|sec|cot)/,
    replacement: '[[0]]\\[[1]]',
    options: 'rmA',
    description: 'Add backslash before trig funcs',
  },

  {
    trigger: /\\(arcsin|sin|arccos|cos|arctan|tan|csc|sec|cot)([A-Za-gi-z])/,
    replacement: '\\[[0]] [[1]]',
    options: 'rmA',
    description:
      'Add space after trig funcs. Skips letter h to allow sinh, cosh, etc.',
  },

  {
    trigger: /\\(sinh|cosh|tanh|coth)([A-Za-z])/,
    replacement: '\\[[0]] [[1]]',
    options: 'rmA',
    description: 'Add space after hyperbolic trig funcs',
  },

  /* -------------------------------------------------------- */
  /*                          Matrix                          */
  /* -------------------------------------------------------- */
  // pmatrix - (...)
  // bmatrix - [...]
  // Bmatrix - {...}
  // vmatrix - |...|
  // Vmatrix - ||...||

  /* ---------------------- Math Block ---------------------- */
  {
    trigger: 'matrix',
    replacement: '\\begin{matrix}\n$0\n\\end{matrix}',
    options: 'MA',
  },
  {
    trigger: /(p|b|v|B|V)mat/,
    replacement: (match) => {
      return `\\begin{${match[1]}matrix}\n$0\n\\end{${match[1]}matrix}`;
    },
    options: 'MA',
    description: 'Matrix notation with parentheses, brackets',
  },
  {
    // n x m Matrix
    // (\d)(\d) --> row & column
    // (y|Y|f|F|h|H|d|D|n|N) --> bracket style
    // lowercase <--> inline math $...$
    // uppercase <--> interline math $$...$$
    trigger: /(\d)(\d)([p|b|B|v|V]?)mat/,
    replacement: (match) => {
      const n = match[1],
        m = match[2],
        c = match[3];

      let arr = [];
      for (let j = 0; j < n; j++) {
        arr[j] = [];
        for (let i = 0; i < m; i++) {
          arr[j][i] = `\${${i + j * m}:${String.fromCharCode(97 + i + j * m)}}`;
        }
      }

      let output = arr.map((el) => el.join(' & ')).join(' \\\\\n');
      output = `\\begin{${c}matrix}\n${output} \n\\end{${c}matrix}`;
      return output;
    },
    options: 'MA',
    description: 'N x M matrix',
  },

  /* ---------------------- Inline Math --------------------- */
  {
    trigger: 'matrix',
    replacement: '\\begin{matrix}$0\\end{matrix}',
    options: 'nA',
  },
  {
    trigger: /(p|b|v|B|V)mat/,
    replacement: (match) => {
      return `\\begin{${match[1]}matrix}\n$0\n\\end{${match[1]}matrix}`;
    },
    options: 'nA',
    description: 'Matrix notation with parentheses, brackets',
  },
  {
    // n x m Matrix
    // (\d)(\d) --> row & column
    // (y|Y|f|F|h|H|d|D|n|N) --> bracket style
    // lowercase <--> inline math $...$
    // uppercase <--> interline math $$...$$
    trigger: /(\d)(\d)([p|b|B|v|V]?)mat/,
    replacement: (match) => {
      const n = match[1],
        m = match[2],
        c = match[3];

      let arr = [];
      for (let j = 0; j < n; j++) {
        arr[j] = [];
        for (let i = 0; i < m; i++) {
          arr[j][i] = `\${${i + j * m}:${String.fromCharCode(97 + i + j * m)}}`;
        }
      }

      let output = arr.map((el) => el.join(' & ')).join(' \\\\ ');
      output = `\\begin{${c}matrix} ${output} \\end{${c}matrix}`;
      return output;
    },
    options: 'nA',
    description: 'N x M matrix',
  },

  {
    trigger: 'array',
    replacement: '\\begin{array}\n$0\n\\end{array}',
    options: 'mA',
  },

  // Misc

  // Automatically convert standalone letters in text to math (except a, A, I).
  // (Un-comment to enable)
  // {trigger: /([^'])\b([B-HJ-Zb-z])\b([\n\s.,?!:'])/, replacement: "[[0]]$[[1]]$[[2]]", options: "tA"},

  // Automatically convert Greek letters in text to math.
  // {trigger: "(${GREEK})([\\n\\s.,?!:'])", replacement: "$\\[[0]]$[[1]]", options: "rtAw"},

  // Automatically convert text of the form "x=2" and "x=n+1" to math.
  // {trigger: /([A-Za-z]=\d+)([\n\s.,?!:'])/, replacement: "$[[0]]$[[1]]", options: "rtAw"},
  // {trigger: /([A-Za-z]=[A-Za-z][+-]\d+)([\n\s.,?!:'])/, replacement: "$[[0]]$[[1]]", options: "tAw"},

  // Snippet replacements can have placeholders.

  // Snippet replacements can also be JavaScript functions.
  // See the documentation for more information.

  // Identity Matrix
  {
    trigger: /iden(\d)/,
    replacement: (match) => {
      const n = match[1];

      let arr = [];
      for (let j = 0; j < n; j++) {
        arr[j] = [];
        for (let i = 0; i < n; i++) {
          arr[j][i] = i === j ? 1 : 0;
        }
      }

      let output = arr.map((el) => el.join(' & ')).join(' \\\\\n');
      output = `\\begin{pmatrix}\n${output}\n\\end{pmatrix}`;
      return output;
    },
    options: 'mA',
    description: 'N x N identity matrix',
  },

  /* -------------------------------------------------------- */
  /*                          Physics                         */
  /* -------------------------------------------------------- */
  { trigger: 'kbt', replacement: 'k_{B}T', options: 'mA' },
  { trigger: 'msun', replacement: 'M_{\\odot}', options: 'mA' },

  /* -------------------------------------------------------- */
  /*                     Quantum Mechanics                    */
  /* -------------------------------------------------------- */
  { trigger: 'dag', replacement: '^{\\dagger}', options: 'mA' },
  { trigger: 'o+', replacement: '\\oplus ', options: 'mA' },
  { trigger: 'ox', replacement: '\\otimes ', options: 'mA', priority: -1 },
  { trigger: 'bra', replacement: '\\bra{$0} $1', options: 'mA' },
  { trigger: 'ket', replacement: '\\ket{$0} $1', options: 'mA' },
  { trigger: 'brk', replacement: '\\braket{ $0 | $1 } $2', options: 'mA' },
  {
    trigger: 'outer',
    replacement: '\\ket{${0:\\psi}} \\bra{${0:\\psi}} $1',
    options: 'mA',
  },

  /* -------------------------------------------------------- */
  /*                         Chemistry                        */
  /* -------------------------------------------------------- */
  { trigger: 'pu', replacement: '\\pu{ $0 }', options: 'mA' },
  { trigger: 'cee', replacement: '\\ce{ $0 }', options: 'mA' },
  { trigger: 'he4', replacement: '{}^{4}_{2}He ', options: 'mA' },
  { trigger: 'he3', replacement: '{}^{3}_{2}He ', options: 'mA' },
  { trigger: 'iso', replacement: '{}^{${0:4}}_{${1:2}}${2:He}', options: 'mA' },
];
