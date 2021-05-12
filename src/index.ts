import defaults from "./defaults";
import parse from "parse-strings-in-object";
import rc from "rc";
import { getLogger } from "log4js";

const config: typeof defaults = parse(rc("lights", defaults));

const logger = getLogger();
logger.level = config.loglevel;

logger.info("started with config", config);
