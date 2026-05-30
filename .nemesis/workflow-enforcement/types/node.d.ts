// Type declarations for Node.js built-ins
declare global {
	var process: {
		env: Record<string, string | undefined>;
		cwd(): string;
		platform: string;
		pid: number;
		ppid: number;
	};

	var require: {
		(id: string): any;
		cache: Record<string, any>;
	};

	var module: {
		exports: any;
	};

	var console: {
		log(...args: any[]): void;
		error(...args: any[]): void;
		warn(...args: any[]): void;
		info(...args: any[]): void;
	};

	var Buffer: {
		from(data: any, encoding?: string): Buffer;
	};
}

export {};
