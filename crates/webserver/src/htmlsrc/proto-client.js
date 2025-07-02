// DO NOT EDIT! This file was automatically generated from proto/messages.proto

console.log('Proto-client.js: Using full JavaScript protobuf client');

/**
 * Full JavaScript protobuf client with binary encoding/decoding
 * Generated from proto definitions: TestMessage
 */

// Protobuf wire types
const WIRE_TYPE_VARINT = 0;
const WIRE_TYPE_FIXED64 = 1;
const WIRE_TYPE_LENGTH_DELIMITED = 2;
const WIRE_TYPE_START_GROUP = 3;
const WIRE_TYPE_END_GROUP = 4;
const WIRE_TYPE_FIXED32 = 5;

// Utility functions for protobuf encoding/decoding
class ProtobufWriter {
    constructor() {
        this.buffer = [];
    }

    writeVarint(value) {
        while (value >= 0x80) {
            this.buffer.push((value & 0xFF) | 0x80);
            value >>>= 7;
        }
        this.buffer.push(value & 0xFF);
    }

    writeTag(fieldNumber, wireType) {
        this.writeVarint((fieldNumber << 3) | wireType);
    }

    writeString(fieldNumber, value) {
        if (value && value.length > 0) {
            const utf8Bytes = new TextEncoder().encode(value);
            this.writeTag(fieldNumber, WIRE_TYPE_LENGTH_DELIMITED);
            this.writeVarint(utf8Bytes.length);
            this.buffer.push(...utf8Bytes);
        }
    }

    writeInt32(fieldNumber, value) {
        if (value !== 0) {
            this.writeTag(fieldNumber, WIRE_TYPE_VARINT);
            this.writeVarint(value);
        }
    }

    getBytes() {
        return new Uint8Array(this.buffer);
    }
}

class ProtobufReader {
    constructor(buffer) {
        this.buffer = new Uint8Array(buffer);
        this.pos = 0;
    }

    readVarint() {
        let result = 0;
        let shift = 0;
        while (this.pos < this.buffer.length) {
            const byte = this.buffer[this.pos++];
            result |= (byte & 0x7F) << shift;
            if ((byte & 0x80) === 0) {
                return result;
            }
            shift += 7;
        }
        throw new Error('Invalid varint');
    }

    readTag() {
        const tag = this.readVarint();
        return {
            fieldNumber: tag >>> 3,
            wireType: tag & 0x7
        };
    }

    readString() {
        const length = this.readVarint();
        const bytes = this.buffer.slice(this.pos, this.pos + length);
        this.pos += length;
        return new TextDecoder().decode(bytes);
    }

    hasMore() {
        return this.pos < this.buffer.length;
    }
}

// Generated message classes
class TestMessage {
    constructor(data = {}) {
        // Initialize with default values
        this.message = data.message || '';
    }

    // Encode this message to protobuf binary format
    encode() {
        const writer = new ProtobufWriter();
        if (this.message) {
            writer.writeString(1, this.message);
        }
        return writer.getBytes();
    }

    // Decode protobuf binary data to create message instance
    static decode(buffer) {
        const reader = new ProtobufReader(buffer);
        const message = new TestMessage();

        while (reader.hasMore()) {
            const tag = reader.readTag();
            switch (tag.fieldNumber) {
                case 1:
                    if (tag.wireType === WIRE_TYPE_LENGTH_DELIMITED) {
                        message.message = reader.readString();
                    }
                    break;
                default:
                    // Skip unknown fields
                    break;
            }
        }

        return message;
    }

    // Convert to JSON for debugging
    toJSON() {
        return {
            message: this.message
        };
    }
}

class ProtoClient {
    constructor() {
        console.log('ProtoClient initialized with messages: TestMessage');
        this.messageTypes = {};
    }

    // Encode message to binary protobuf format
    encode(message) {
        if (!message || typeof message.encode !== 'function') {
            throw new Error('Message must have an encode method');
        }
        return message.encode();
    }

    // Decode binary data based on message type
    decode(messageClass, data) {
        if (!messageClass || typeof messageClass.decode !== 'function') {
            throw new Error('Message class must have a decode method');
        }
        return messageClass.decode(data);
    }

    // Create a new message instance
    create(messageType, data = {}) {
        const MessageClass = this.messageTypes[messageType];
        if (!MessageClass) {
            throw new Error(`Unknown message type: ${messageType}`);
        }
        return new MessageClass(data);
    }

    // Check if real protobuf is available
    isPlaceholder() {
        return false; // This is a full protobuf implementation
    }

    // Get list of available message types
    getMessageTypes() {
        return Object.keys(this.messageTypes);
    }
}

// Register message types
const protoClient = new ProtoClient();
protoClient.messageTypes['TestMessage'] = TestMessage;

// Export for CommonJS
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ProtoClient, protoClient, TestMessage };
}

// Export for ES6 modules
if (typeof window !== 'undefined') {
    window.ProtoClient = ProtoClient;
    window.protoClient = protoClient;
    window.TestMessage = TestMessage;
}
