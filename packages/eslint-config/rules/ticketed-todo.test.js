import { javascriptRuleTester } from './rule-tester.js';
import { ticketedTodo } from './ticketed-todo.js';

// Los marcadores se arman por concatenacion para que este fichero no viole la
// misma regla que prueba.
const pending = 'TO' + 'DO';
const broken = 'FIX' + 'ME';
const odd = 'XX' + 'X';
const workaround = 'HA' + 'CK';

const missingTicket = [{ messageId: 'missingTicket' }];

javascriptRuleTester().run('ticketed-todo', ticketedTodo, {
  valid: [
    `// ${pending}(#123): mover esto al motor`,
    `// ${pending}(RAM-45): mover esto al motor`,
    `/* ${broken}(#9) cuando exista el endpoint */`,
    `// ${odd}(RAM-1): caso raro de rebase`,
    `// ${workaround}(#77): rodeo hasta que salga la version nueva`,
    '// esto lo hace todo de golpe',
    'const nota = 1;',
  ],
  invalid: [
    { code: `// ${pending}: arreglar esto`, errors: missingTicket },
    { code: `// ${pending}(ABC-12): otro tablero no vale`, errors: missingTicket },
    { code: `// ${pending}(123): sin almohadilla no vale`, errors: missingTicket },
    { code: `/* ${broken} */`, errors: missingTicket },
    { code: `// ${odd} caso raro`, errors: missingTicket },
    { code: `// ${workaround} rapido`, errors: missingTicket },
  ],
});
